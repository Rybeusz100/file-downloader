pub enum UrlType {
    WeTransfer,
}

pub fn check_url(url: &str) -> Option<UrlType> {
    if url.starts_with("https://we.tl/") || url.starts_with("https://wetransfer.com/") {
        return Some(UrlType::WeTransfer);
    }
    None
}
