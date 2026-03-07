use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc};

type ValueStore = RwLock<HashMap<String, CacheEntry>>;
type MetadataStore = RwLock<HashMap<String, CacheMetadata>>;

const DEFAULT_TTL_SECONDS: i64 = 60;

#[derive(Clone, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct PutRequest {
    value: String,
    ttl: Option<u64>
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

struct AppState {
    values: ValueStore,
    metadata: MetadataStore,
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
    let now = Utc::now();
    // let expires_at = body.ttl.map(|ttl| now + chrono::Duration::seconds(ttl as i64));

    let ttl = body.ttl.unwrap_or(DEFAULT_TTL_SECONDS as u64);
    let expires_at = Some(now + chrono::Duration::seconds(ttl as i64));

    {
        let mut values = state.values.write().unwrap();

        values.insert(
            key.clone(),
            CacheEntry {
                value: body.value.clone(),
            },
        );
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
                expires_at,
            },
        );
    }

    HttpResponse::Ok().json(ApiResponse { status: "ok".into() })
}

async fn get_key(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> impl Responder {

    let key = key.into_inner();

    let values = state.values.read().unwrap();

    if let Some(entry) = values.get(&key) {

        {
            let mut metadata = state.metadata.write().unwrap();

            if let Some(meta) = metadata.get_mut(&key) {

                if let Some(expiry) = meta.expires_at {
                    if Utc::now() > expiry {
                        drop(values);
                        drop(metadata);

                        state.values.write().unwrap().remove(&key);
                        state.metadata.write().unwrap().remove(&key);

                        return HttpResponse::NotFound().json(ApiResponse { status: "key_expired".into() });
                    }
                }

                meta.last_accessed_at = Utc::now();
                meta.frequency += 1;
            }
        }

        HttpResponse::Ok().json(GetResponse {
            key,
            value: entry.value.clone(),
        })

    } else {
        HttpResponse::NotFound().json(ApiResponse { status: "key_not_found".into() })
    }
}

async fn delete_key(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> impl Responder {

    let key = key.into_inner();

    {
        let mut values = state.values.write().unwrap();
        values.remove(&key);
    }

    {
        let mut metadata = state.metadata.write().unwrap();
        metadata.remove(&key);
    }

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
        HttpResponse::NotFound().json(ApiResponse { status: "key_not_found".into() })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let state = web::Data::new(AppState {
        values: RwLock::new(HashMap::new()),
        metadata: RwLock::new(HashMap::new()),
    });

    println!("VelocityCache server starting on http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(
                web::scope("/velocitycache")
                    .route("/health", web::get().to(health))
                    .route("/cache/{key}", web::put().to(put_key))
                    .route("/cache/{key}", web::get().to(get_key))
                    .route("/cache/{key}", web::delete().to(delete_key))
                    .route("/cache/{key}/metadata", web::get().to(get_metadata)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
