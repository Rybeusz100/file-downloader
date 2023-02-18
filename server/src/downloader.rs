use std::{fs, path::Path};

use crate::DOWNLOAD_DIR;

pub mod wetransfer;
pub mod youtube;

struct FileInfo {
    name: String,
    path: String,
}

fn get_file_info(intended_name: &str, username: &str) -> FileInfo {
    let dir = DOWNLOAD_DIR.to_owned() + username + "/";
    create_dir(&dir).unwrap();
    let (file_name, extension) = get_name_extension(intended_name);
    let dot_extension = match extension {
        Some(e) => ".".to_owned() + e,
        None => "".to_owned(),
    };
    let mut final_name = intended_name.to_owned();
    let base_path = Path::new(&dir);
    let mut i: u8 = 1;
    while base_path.join(&final_name).exists() {
        final_name = file_name.to_owned() + " (" + &(i.to_string()) + ")" + &dot_extension;
        i += 1;
    }

    FileInfo {
        name: final_name.clone(),
        path: dir + &final_name,
    }
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

fn create_dir(dir: &str) -> std::io::Result<()> {
    match fs::metadata(dir) {
        Ok(_) => (),
        Err(_) => fs::create_dir_all(dir)?,
    }
    Ok(())
}
