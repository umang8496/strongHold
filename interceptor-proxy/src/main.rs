// src/main.rs

mod config;
mod error;
mod interceptor;
mod mutation;
mod proxy;
mod resolver;

use clap::Parser;
use config::Config;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    if let Err(err) = config.validate() {
        eprintln!("[proxy-circuit configuration error]: {err}");
        std::process::exit(1);
    }

    let bind_addr = config.socket_addr()?;
    let shared_config = Arc::new(config);
    
    // Initialize the shared upstream HTTP connection pool
    let upstream_client = proxy::build_upstream_client();

    let listener = TcpListener::bind(bind_addr).await?;
    println!(
        "⚡ proxy-circuit operational on http://{} [Mode: {:?}]",
        bind_addr, shared_config.mode
    );

    if let Some(ref upstream) = shared_config.upstream {
        println!(" Forwarding upstream target: {upstream}");
    }

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let task_config = Arc::clone(&shared_config);
        let task_client = upstream_client.clone();

        // Wrap the standard Tokio TCP stream in Hyper's TokioIo adapter
        let io = TokioIo::new(stream);

        tokio::spawn(async move {
            // Bind the socket to Hyper's auto-engine (supports both HTTP/1.1 and HTTP/2)
            let service = service_fn(move |req| {
                proxy::handle_request(req, Arc::clone(&task_config), task_client.clone())
            });

            // auto::Builder supports auto-negotiation between HTTP/1 and HTTP/2
            if let Err(err) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, service)
                .await
            {
                // Connection resets or abrupt client disconnects are common in networking
                eprintln!("[Connection closed for {peer_addr}]: {err}");
            }
        });
    }
}