use std::collections::HashMap;
use std::path::Path;

pub fn download_file(
    url: &str,
    file: &Path,
    headers: &HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let agent = ureq::builder()
        .try_proxy_from_env(true)
        .build();
    let mut body = agent.get(url);
    for (key, value) in headers {
        body = body.set(key, value);
    }
    println!("Body {:?}", body);
    let res = body.call()?;
    println!("Res {}", res.status());
    std::io::copy(&mut res.into_reader(), &mut std::fs::File::create(file)?)?;
    Ok(())
}

pub fn upload_file(
    url: &str,
    file: &Path,
    headers: &HashMap<String, String>,
) -> Result<(), anyhow::Error> {
    let agent = ureq::builder()
        .build();
    let mut body = agent.put(url);
    for (key, value) in headers {
        body = body.set(key, value);
    }
    let file = std::fs::File::open(file)?;
    match body.send(file) {
        Ok(res) => {
            if res.status() != 201 {
                println!(
                    "Failed to upload file: {} {}",
                    res.status(),
                    res.status_text()
                );
            }
        }
        Err(e) => {
            println!("Error: {e}");
        }
    }
    Ok(())
}
