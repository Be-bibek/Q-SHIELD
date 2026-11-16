use qshield_api::{create_router, state::AppState};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,qshield=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("QSHIELD_API_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port: u16 = std::env::var("QSHIELD_API_PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .unwrap_or(8080);

    let state = AppState::new();
    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    println!("=======================================================");
    println!("   Q-SHIELD REST API SERVER LISTENING ON http://{}", addr);
    println!("   Endpoints available at /api/v1/*                    ");
    println!("=======================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
