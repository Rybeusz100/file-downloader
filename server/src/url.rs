pub enum UrlType {
    WeTransfer,
    YouTube,
}

pub fn check_url(url: &str) -> Option<UrlType> {
    if url.starts_with("https://we.tl/") || url.starts_with("https://wetransfer.com/") {
        return Some(UrlType::WeTransfer);
    } else if url.starts_with("https://youtu.be/")
        || url.starts_with("https://www.youtube.com/")
        || url.starts_with("https://youtube.com/")
    {
        return Some(UrlType::YouTube);
    }
    None
}
