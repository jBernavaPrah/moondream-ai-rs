use base64::{Engine as _, engine::general_purpose};
use image::ImageFormat;
use moondream::{CaptionLength, MoonDream};
use std::io::Cursor;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "INFO");
        }
    }
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    info!("Caption started");

    let path = "moondream/examples/example.jpeg";

    let image = image::open(path)?;
    let format = ImageFormat::from_path(path)?;

    let mut data: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut data), format)?;

    let response =
        MoonDream::remote(std::env::var("MOONDREAM_API_KEY").expect("MOONDREAM_API_KEY not set"))
            .caption(
                format!(
                    "data:{};base64,{}",
                    format.to_mime_type(),
                    general_purpose::STANDARD.encode(&data)
                ),
                Some(CaptionLength::Normal),
            )
            .await?;

    info!("{:#?}", response);

    Ok(())
}
