use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufWriter},
};

use crate::DownloadResult;

use super::get_file_info;

#[derive(Serialize)]
struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<String>,
    pub security_hash: String,
    pub intent: String,
}

#[derive(Deserialize)]
struct FinalResponse {
    direct_link: String,
}

pub async fn download(
    mut url: String,
    username: &str,
) -> Result<DownloadResult, Box<dyn Error + Send + Sync>> {
    if url.starts_with("https://we.tl/") {
        let resp = reqwest::get(&url).await?;
        url = "https://wetransfer.com".to_owned() + resp.url().path();
    }

    let mut params = url.split('/').collect::<Vec<&str>>();
    if params.len() < 6 {
        return Err("Invalid URL".into());
    }
    params.drain(0..4);

    let mut body = RequestBody {
        recipient_id: None,
        security_hash: "".to_owned(),
        intent: "entire_transfer".to_owned(),
    };

    let transfer_id: String;

    match params.len() {
        2 => {
            transfer_id = params[0].to_owned();
            body.security_hash = params[1].split('?').collect::<Vec<&str>>()[0].to_owned();
        }
        3 => {
            transfer_id = params[0].to_owned();
            body.recipient_id = Some(params[1].to_owned());
            body.security_hash = params[2].split('?').collect::<Vec<&str>>()[0].to_owned();
        }
        _ => return Err("Invalid URL".into()),
    }

    let client = reqwest::Client::new();
    let resp = client
        .post("https://wetransfer.com/api/v4/transfers/".to_owned() + &transfer_id + "/download")
        .json(&body)
        .send()
        .await?;

    let final_response: FinalResponse = serde_json::from_str(&resp.text().await?)?;
    let file_response = reqwest::get(&final_response.direct_link).await?;
    let original_file_name = file_response
        .url()
        .path()
        .split('/')
        .last()
        .unwrap()
        .to_owned()
        .replace("%20", " ");
    let file_info = get_file_info(&original_file_name, username);
    let mut file = BufWriter::new(File::create(&file_info.path).await?);
    let mut stream = file_response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }
    file.flush().await?;
    let file_size = fs::metadata(Path::new(&file_info.path)).await?.len();
    Ok(DownloadResult {
        file_name: file_info.name,
        file_size,
    })
}
