use anyhow::{Context, bail};
use auto_launch::AutoLaunch;
use minreq::get;
use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use std::{any, env};
use std::path::PathBuf;
use anyhow::anyhow;
use std::process::Command;
use chrono::prelude::*;
use chrono_tz::Europe::Paris;

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

fn update_wallpaper() -> anyhow::Result<()> {
    let now_paris = Utc::now().with_timezone(&Paris);

    let url = format!(
        "https://data.skaping.com/amboise-quais-de-loire/photo/{}/{:02}/{:02}/{:02}-{:02}.jpg",
        now_paris.year(),
        now_paris.month(),
        now_paris.day(),
        now_paris.hour(),
        (now_paris.minute() / 10) * 10
    );

    println!("Downloading image from: {}", url);
    let mut attempts = 0;
    let rep = loop {
        let rep = get(&url).send().context("Failed to send request")?;

        if rep.status_code == 200 {
            break rep;
        }

        if rep.status_code == 403 {
            attempts += 1;
            if attempts >= 10 {
                bail!("Received 403 Forbidden after {} attempts", attempts);
            }
            println!("Received 403 Forbidden, retrying in 60 seconds...");
            sleep(Duration::from_secs(60));
            continue;
        }

        bail!("Failed to download image: HTTP {}", rep.status_code);
    };

    let body = rep.as_bytes();
    let path = std::env::current_dir().unwrap().join("image.jpg");
    write(&path, body).map_err(|e| anyhow!("Failed to write image to file: {}", e))?;
    println!("Image downloaded to: {}", path.display());

    set_wallpaper(path.clone())
}

fn main() {
    let app_name = "realtime-background";
    let app_path = env::current_exe().unwrap();
    let auto = AutoLaunch::new(app_name, app_path.to_str().unwrap(), &[] as &[&str]);
    auto.enable().expect("Failed to enable auto-launch");

    loop {
        match update_wallpaper() {
            Ok(_) => {
                println!("Wallpaper updated successfully.");
            }
            Err(e) => {
                eprintln!("Error updating wallpaper: {}", e);
            }
        }

        println!("Sleeping 10 minutes");
        sleep(Duration::from_secs(600));
    }
}
