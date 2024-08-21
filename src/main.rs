use anyhow::Result;
use simple_redis::Backend;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis_server is Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let backend = Backend::new();

    loop {
        let (socket, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            match simple_redis::network::stream_handler(socket, cloned_backend).await {
                Ok(_) => {
                    info!("Connection from {} is handled successfully", raddr);
                }
                Err(e) => warn!("Error: {:?}", e),
            }
        });
    }
}
