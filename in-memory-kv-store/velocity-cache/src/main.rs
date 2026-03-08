use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

type ValueStore = RwLock<HashMap<String, CacheEntry>>;
type MetadataStore = RwLock<HashMap<String, CacheMetadata>>;

const DEFAULT_TTL_SECONDS: u64 = 60;
const WAL_FILE: &str = "velocitycache.wal";

#[derive(Clone)]
struct CacheEntry {
    value: String,
}

#[derive(Clone, Serialize)]
struct CacheMetadata {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
    frequency: u64,
    size: usize,
    ttl: u64,
}

#[derive(Deserialize)]
struct PutRequest {
    value: String,
    ttl: Option<u64>,
}

#[derive(Serialize)]
struct GetResponse {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
}

#[derive(Serialize)]
struct StatsResponse {
    total_requests: u64,
    hits: u64,
    misses: u64,
    sets: u64,
    deletes: u64,
    total_keys: usize,
}

enum WalEntry {
    Put(String, String, u64),
    Delete(String),
}

struct CacheStats {
    total_requests: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
    sets: AtomicU64,
    deletes: AtomicU64,
}

struct AppState {
    values: ValueStore,
    metadata: MetadataStore,
    stats: CacheStats,
    wal_sender: mpsc::Sender<WalEntry>,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json("VelocityCache is running")
}

async fn put_key(
    state: web::Data<AppState>,
    key: web::Path<String>,
    body: web::Json<PutRequest>,
) -> impl Responder {

    let key = key.into_inner();
    let ttl = body.ttl.unwrap_or(DEFAULT_TTL_SECONDS);
    let now = Utc::now();

    let _ = state
        .wal_sender
        .send(WalEntry::Put(key.clone(), body.value.clone(), ttl))
        .await;

    {
        let mut values = state.values.write().unwrap();
        values.insert(key.clone(), CacheEntry { value: body.value.clone() });
    }

    {
        let mut metadata = state.metadata.write().unwrap();

        metadata.insert(
            key,
            CacheMetadata {
                created_at: now,
                updated_at: now,
                last_accessed_at: now,
                frequency: 0,
                size: body.value.len(),
                ttl,
            },
        );
    }

    state.stats.sets.fetch_add(1, Ordering::Relaxed);
    state.stats.total_requests.fetch_add(1, Ordering::Relaxed);

    HttpResponse::Ok().json(ApiResponse { status: "ok".into() })
}

async fn get_key(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> impl Responder {

    let key = key.into_inner();
    state.stats.total_requests.fetch_add(1, Ordering::Relaxed);

    let values = state.values.read().unwrap();

    if let Some(entry) = values.get(&key) {

        {
            let mut metadata = state.metadata.write().unwrap();

            if let Some(meta) = metadata.get_mut(&key) {

                let expiry_time =
                    meta.last_accessed_at + chrono::Duration::seconds(meta.ttl as i64);

                if Utc::now() > expiry_time {

                    drop(values);
                    drop(metadata);

                    state.values.write().unwrap().remove(&key);
                    state.metadata.write().unwrap().remove(&key);

                    state.stats.misses.fetch_add(1, Ordering::Relaxed);

                    return HttpResponse::NotFound()
                        .json(ApiResponse { status: "key_expired".into() });
                }

                meta.last_accessed_at = Utc::now();
                meta.frequency += 1;
            }
        }

        state.stats.hits.fetch_add(1, Ordering::Relaxed);

        HttpResponse::Ok().json(GetResponse {
            key,
            value: entry.value.clone(),
        })

    } else {

        state.stats.misses.fetch_add(1, Ordering::Relaxed);

        HttpResponse::NotFound()
            .json(ApiResponse { status: "key_not_found".into() })
    }
}

async fn delete_key(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> impl Responder {

    let key = key.into_inner();

    let _ = state
        .wal_sender
        .send(WalEntry::Delete(key.clone()))
        .await;

    {
        let mut values = state.values.write().unwrap();
        values.remove(&key);
    }

    {
        let mut metadata = state.metadata.write().unwrap();
        metadata.remove(&key);
    }

    state.stats.deletes.fetch_add(1, Ordering::Relaxed);
    state.stats.total_requests.fetch_add(1, Ordering::Relaxed);

    HttpResponse::Ok().json(ApiResponse { status: "deleted".into() })
}

async fn get_metadata(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> impl Responder {

    let key = key.into_inner();

    let metadata = state.metadata.read().unwrap();

    if let Some(meta) = metadata.get(&key) {
        HttpResponse::Ok().json(meta)
    } else {
        HttpResponse::NotFound()
            .json(ApiResponse { status: "key_not_found".into() })
    }
}

async fn stats(state: web::Data<AppState>) -> impl Responder {

    let values = state.values.read().unwrap();

    let response = StatsResponse {
        total_requests: state.stats.total_requests.load(Ordering::Relaxed),
        hits: state.stats.hits.load(Ordering::Relaxed),
        misses: state.stats.misses.load(Ordering::Relaxed),
        sets: state.stats.sets.load(Ordering::Relaxed),
        deletes: state.stats.deletes.load(Ordering::Relaxed),
        total_keys: values.len(),
    };

    HttpResponse::Ok().json(response)
}

async fn cleanup_expired_keys(state: web::Data<AppState>) {

    loop {

        sleep(Duration::from_secs(10)).await;

        let now = Utc::now();
        let mut expired = Vec::new();

        {
            let metadata = state.metadata.read().unwrap();

            for (key, meta) in metadata.iter() {

                let expiry_time =
                    meta.last_accessed_at + chrono::Duration::seconds(meta.ttl as i64);

                if now > expiry_time {
                    expired.push(key.clone());
                }
            }
        }

        if !expired.is_empty() {

            let mut values = state.values.write().unwrap();
            let mut metadata = state.metadata.write().unwrap();

            for key in expired {
                values.remove(&key);
                metadata.remove(&key);
            }
        }
    }
}

async fn wal_writer(mut receiver: mpsc::Receiver<WalEntry>) {

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(WAL_FILE)
        .unwrap();

    while let Some(entry) = receiver.recv().await {

        match entry {

            WalEntry::Put(k, v, ttl) => {
                writeln!(file, "PUT {} {} {}", k, v, ttl).unwrap();
            }

            WalEntry::Delete(k) => {
                writeln!(file, "DELETE {}", k).unwrap();
            }
        }

        file.sync_data().unwrap();
    }
}

fn replay_wal(values: &ValueStore, metadata: &MetadataStore) {

    if let Ok(contents) = read_to_string(WAL_FILE) {

        for line in contents.lines() {

            let parts: Vec<&str> = line.split_whitespace().collect();

            match parts[0] {

                "PUT" => {

                    if parts.len() < 4 {
                        continue;
                    }

                    let key = parts[1].to_string();
                    let value = parts[2].to_string();
                    let ttl: u64 = parts[3].parse().unwrap();

                    let now = Utc::now();

                    values.write().unwrap().insert(
                        key.clone(),
                        CacheEntry { value: value.clone() },
                    );

                    metadata.write().unwrap().insert(
                        key,
                        CacheMetadata {
                            created_at: now,
                            updated_at: now,
                            last_accessed_at: now,
                            frequency: 0,
                            size: value.len(),
                            ttl,
                        },
                    );
                }

                "DELETE" => {

                    let key = parts[1];

                    values.write().unwrap().remove(key);
                    metadata.write().unwrap().remove(key);
                }

                _ => {}
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init();

    let (wal_sender, wal_receiver) = mpsc::channel(10000);

    let state = web::Data::new(AppState {
        values: RwLock::new(HashMap::new()),
        metadata: RwLock::new(HashMap::new()),
        wal_sender,
        stats: CacheStats {
            total_requests: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            sets: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
        },
    });

    replay_wal(&state.values, &state.metadata);

    let cleaner_state = state.clone();

    tokio::spawn(async move {
        cleanup_expired_keys(cleaner_state).await;
    });

    tokio::spawn(async move {
        wal_writer(wal_receiver).await;
    });

    info!("VelocityCache starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(
                web::scope("/velocitycache")
                    .route("/health", web::get().to(health))
                    .route("/cache/{key}", web::put().to(put_key))
                    .route("/cache/{key}", web::get().to(get_key))
                    .route("/cache/{key}", web::delete().to(delete_key))
                    .route("/cache/{key}/metadata", web::get().to(get_metadata))
                    .route("/stats", web::get().to(stats)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
