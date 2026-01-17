use anyhow::{Context, bail, anyhow, Result as AnyResult};
use auto_launch::AutoLaunch;
use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn is_gsettings_available() -> bool {
    env::var("PATH")
        .ok()
        .and_then(|paths| {
            for path in env::split_paths(&paths) {
                let full_path = path.join("gsettings");
                if full_path.is_file() {
                    return Some(true);
                }
            }
            None
        })
        .unwrap_or(false)
}

fn set_gnome_background(path: PathBuf) -> anyhow::Result<()> {
    let uri = format!("file://{}", path.canonicalize()?.display());

    let commands = [
        ("org.gnome.desktop.background", "picture-uri", &uri),
        ("org.gnome.desktop.background", "picture-uri-dark", &uri),
    ];

    for (schema, key, value) in commands.iter() {
        let status = Command::new("gsettings")
            .args(["set", schema, key, value])
            .status()
            .map_err(|e| anyhow!("Failed to execute gsettings command: {}", e))?;

        if !status.success() {
            bail!("Failed to set gsettings for {}: {}", schema, key);
        }
    }

    Ok(())
}


fn set_wallpaper(path: PathBuf) -> anyhow::Result<()> {
    if is_gsettings_available() {
        set_gnome_background(path)
    } else {
        let path = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
        wallpaper::set_from_path(path).map_err(|e| anyhow!("Failed to set wallpaper: {}", e))
    }
}

fn update_image(last_date: String, last_etag: String) -> AnyResult<Option<(String, String, String)>> {
    let rep = minreq::get("https://amboise.dera.page")
        .with_header("If-Modified-Since", last_date.as_str())
        .with_header("If-None-Match", last_etag.as_str())
        .send()
        .context("Failed to send request")?;

    if rep.status_code == 304 {
        return Ok(None);
    }

    if rep.status_code != 200 {
        bail!("Failed to download image: HTTP {}", rep.status_code);
    }

    let body = rep.as_bytes();
    let path = std::env::current_dir().unwrap().join("image.jpg");
    write(&path, body).map_err(|e| anyhow!("Failed to write image to file: {}", e))?;

    let last_modified = rep
        .headers
        .get("last-modified")
        .ok_or_else(|| anyhow!("Missing Last-Modified header"))?
        .to_string();

    let etag = rep
        .headers
        .get("etag")
        .ok_or_else(|| anyhow!("Missing ETag header"))?
        .to_string();

    Ok(Some((path.to_string_lossy().to_string(), last_modified, etag)))
}

fn main() {
    let app_name = "realtime-background";
    let app_path = env::current_exe().unwrap();
    let auto = AutoLaunch::new(app_name, app_path.to_str().unwrap(), &[] as &[&str]);
    auto.enable().expect("Failed to enable auto-launch");

    let mut last_modified = String::new();
    let mut etag = String::new();

    loop {
        match update_image(last_modified.clone(), etag.clone()) {
            Ok(None) => (),
            Ok(Some((path, new_last_modified, new_etag))) => {
                println!("Image updated successfully! ({path})");

                last_modified = new_last_modified;
                etag = new_etag;

                if let Err(e) = set_wallpaper(PathBuf::from(path)) {
                    eprintln!("Failed to set wallpaper: {}", e);
                }
            },
            Err(e) => eprintln!("Error updating wallpaper: {}", e),
        }

        println!("Sleeping 60 seconds");
        sleep(Duration::from_secs(60));
    }
}
