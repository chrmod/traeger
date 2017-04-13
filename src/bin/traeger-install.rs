use std::fs::File;
use std::path::{PathBuf, Path};
use std::io;
use std::io::Write;
use std::env;

fn main() {
    let path = match get_manifest_path() {
        Some(path) => path,
        None => panic!("cannot find path"),
    };
    let file_path = path.join("traeger.json");
    match File::open(&file_path) {
        Ok(_) => println!("manifest already extists"),
        Err(_) => {
            create_manifest(&file_path).unwrap();
            println!("manifest created at path: {}", file_path.display());
        },
    };
}

fn create_manifest<P: AsRef<Path>>(file_path: P) -> io::Result<File> {
    let mut file = File::create(&file_path).unwrap();
    let template = include_str!("../../static/native-host-manifest.json");
    file.write_all(template.as_bytes())?;
    return Ok(file);
}

fn get_manifest_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    let path = match env::home_dir() {
        Some(path) => path.join(".config/google-chrome/NativeMessagingHosts"),
        None => return None,
    };

    #[cfg(target_os = "macos")]
    let path = match env::home_dir() {
        Some(path) => path.join("Library/Application Support/Chromium/NativeMessagingHosts"),
        None => return None,
    };

    #[cfg(target_os = "windows")]
    let path = env::current_exe().unwrap();

    return Some(path);
}
