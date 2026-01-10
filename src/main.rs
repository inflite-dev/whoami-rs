use axum::routing::{Router, get, post, put};
use bytes::Bytes;
use std::net::SocketAddr;

#[axum::debug_handler]
async fn hello() -> Bytes {
    // No exclamation or capitalization here for our hello world here;
    // the canonical version of this program is not so excited to be alive.
    // https://en.wikipedia.org/wiki/%22Hello,_World!%22_program
    Bytes::from("hello, world")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/hello", post(hello))
        .route("/hello", put(hello));

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);

    if let Err(err) = server.await {
        eprintln!("server error: {err}");
    }
    Ok(())
}
