use async_stream::try_stream;
use axum::extract::TypedHeader;
use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use headers::authorization::Bearer;
use headers::Authorization;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::{convert::Infallible, time::Duration};

use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_sse=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let static_files_service = ServeDir::new(assets_dir).append_index_html_on_directories(true);

    // build our application with a route
    let app = Router::new()
        .fallback_service(static_files_service)
        .route("/async-sse", get(sse_handler))
        // .route("/async-sse", get(sse_handler2))
        .layer(TraceLayer::new_for_http());

    // run it

    axum::Server::bind(&SocketAddr::from_str("127.0.0.1:3000").unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn sse_handler(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{:?}` connected", bearer.token());

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let mut curr = 0;
    let stream = try_stream! {
        loop {
            curr  += 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            yield Event::default().data(curr.to_string())
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::default()
            .interval(Duration::from_secs(1))
            .text("keep-alive_text"),
    )
}
