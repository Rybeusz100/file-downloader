use std::path::Path;

pub mod wetransfer;
pub mod youtube;

fn get_file_name(dir: &str, intended_name: &str) -> String {
    let (file_name, extension) = get_name_extension(intended_name);
    let dot_extension = match extension {
        Some(e) => ".".to_owned() + e,
        None => "".to_owned(),
    };
    let mut final_name = intended_name.to_owned();
    let base_path = Path::new(dir);
    let mut i: u8 = 1;
    while base_path.join(&final_name).exists() {
        final_name = file_name.to_owned() + " (" + &(i.to_string()) + ")" + &dot_extension;
        i += 1;
    }
    final_name
}

fn get_name_extension(path: &str) -> (&str, Option<&str>) {
    let path = Path::new(path);
    let file_name = match path.file_stem() {
        Some(name) => name.to_str().unwrap_or("downloaded"),
        None => "downloaded",
    };
    let file_extension = match path.extension() {
        Some(extension) => extension.to_str(),
        None => None,
    };
    (file_name, file_extension)
}
