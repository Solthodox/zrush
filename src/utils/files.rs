use std::fs::File;
use std::io::ErrorKind;
use std::io::{stdin, Read, Write};
use std::path::Path;
use std::process;
pub fn read_from_file(path: &str, file_name: &str) -> Result<String, ()> {
    let file_path = Path::new(&path).join(file_name);
    let mut content = String::new();
    match File::open(file_path) {
        Ok(mut file) => {
            let _ = file.read_to_string(&mut content);
            return Ok(content);
        }
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Error: could not read from {path}{file_name}: not found");
                }
                _ => {
                    eprintln!("Error: could not read from {path}{file_name}: unknown reason");
                }
            }
            process::exit(1)
        }
    }
}

pub fn write_to_file(path: &str, file_name: &str, content: &String) -> Result<(), ()> {
    let folder_path = Path::new(path);
    let file_path = folder_path.join(file_name);

    std::fs::create_dir_all(&folder_path).unwrap();

    let mut file = File::create(file_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    Ok(())
}
