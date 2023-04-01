use axum::{
    body::BoxBody,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::StatusCode;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root_get));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_get() -> Response<BoxBody> {
    match get_catscii().await {
        Ok(art) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            art,
        )
            .into_response(),
        Err(e) => {
            println!("Something went wrong: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
        }
    }
}
async fn get_cat_image_url() -> color_eyre::Result<String> {
    let api_url = "https://api.thecatapi.com/v1/images/search";
    let res = reqwest::get(api_url).await?;
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

async fn get_cat_image_bytes() -> color_eyre::Result<Vec<u8>> {
    let cat_url = get_cat_image_url().await.unwrap();
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

async fn get_catscii() -> color_eyre::Result<String> {
    let image_bytes = get_cat_image_bytes().await.unwrap();
    let image = image::load_from_memory(&image_bytes)?;
    let ascii_art = artem::convert(
        image,
        artem::options::OptionBuilder::new()
            .target(artem::options::TargetType::HtmlFile(true, true))
            .build(),
    );
    Ok(ascii_art)
}
