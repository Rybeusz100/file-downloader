use std::fs;
use std::{error::Error, path::Path};

use crate::{downloader::get_file_name, DownloadResult, DOWNLOAD_DIR};

pub async fn download(url: String) -> Result<DownloadResult, Box<dyn Error + Send + Sync>> {
    let file_name = get_file_name(DOWNLOAD_DIR, "YouTube video.mp4");
    let file_path = DOWNLOAD_DIR.to_owned() + &file_name;
    let video = rustube::Video::from_url(&reqwest::Url::parse(&url)?).await?;

    video
        .best_quality()
        .ok_or("Error downloading YouTube video")?
        .download_to(&file_path)
        .await?;

    let file_size = fs::metadata(Path::new(&file_path))?.len() as usize;
    Ok(DownloadResult {
        file_name,
        file_size,
    })
}
