use std::str::FromStr;

use axum::{
    body::BoxBody,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct ServerState {
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let state = ServerState {
        client: Default::default(),
    };
    let filter = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info"))
        .expect("RUST_LOG should be a valid tracing filter");
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .json()
        .finish()
        .with(filter)
        .init();

    let app = Router::new().route("/", get(root_get)).with_state(state);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_get(State(state): State<ServerState>) -> Response<BoxBody> {
    match get_catscii(state).await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            println!("Something went wrong: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
        }
    }
}
async fn get_cat_image_url(state: ServerState) -> color_eyre::Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let res = state.client.get(api_url).send().await?;
    if !res.status().is_success() {
        return Err(color_eyre::eyre::eyre!(
            "The Cat API returned HTTP {}",
            res.status()
        ));
    }

    #[derive(Deserialize, Clone)]
    struct CatImage {
        url: String,
    }
    let mut images: Vec<CatImage> = res.json().await?;
    // this syntax is new in Rust 1.65
    let Some(image) = images.pop() else {
        return Err(color_eyre::eyre::eyre!("The Cat API returned no images"));
    };
    Ok(image.url)
}

async fn get_cat_image_bytes(state: ServerState) -> color_eyre::Result<Vec<u8>> {
    let cat_url = get_cat_image_url(state).await.unwrap();
    let client = reqwest::Client::new();
    Ok(client
        .get(cat_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

async fn get_catscii(state: ServerState) -> color_eyre::Result<String> {
    let image_bytes = get_cat_image_bytes(state).await.unwrap();
    let image = image::load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        artem::options::OptionBuilder::new()
            .target(artem::options::TargetType::HtmlFile(true, true))
            .build(),
    );
    Ok(ascii_art)
}
