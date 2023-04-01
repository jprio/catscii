use serde::Deserialize;

#[tokio::main]
async fn main() {
    let image = get_cat_image_bytes().await.unwrap();
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
