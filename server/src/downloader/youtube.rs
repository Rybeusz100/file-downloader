use std::{error::Error, path::Path};
use tokio::fs;

use crate::{downloader::get_file_info, DownloadResult};

pub async fn download(
    url: String,
    username: &str,
) -> Result<DownloadResult, Box<dyn Error + Send + Sync>> {
    let file_info = get_file_info("YouTube video.mp4", username);
    let video = rustube::Video::from_url(&reqwest::Url::parse(&url)?).await?;

    video
        .best_quality()
        .ok_or("Error downloading YouTube video")?
        .download_to(&file_info.path)
        .await?;

    let file_size = fs::metadata(Path::new(&file_info.path)).await?.len();
    Ok(DownloadResult {
        file_name: file_info.name,
        file_size,
    })
}
