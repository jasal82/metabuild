use std::fs::File;
use std::io::Cursor;
use std::path::Path;

pub fn download_file(url: &str, file: &Path) -> Result<(), anyhow::Error> {
    let response = reqwest::blocking::get(url)?;
    let mut f = File::create(file)?;
    let mut content = Cursor::new(response.bytes()?);
    std::io::copy(&mut content, &mut f)?;
    Ok(())
}