use chrono_tz::Tz;
use minreq::get;
use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{Context, bail, anyhow, Result as AnyResult};
use chrono::prelude::*;
use chrono_tz::Europe::Paris;

fn update_wallpaper(date: DateTime<Tz>) -> AnyResult<()> {
    let url = format!(
        "https://data.skaping.com/amboise-quais-de-loire/photo/{}/{:02}/{:02}/{:02}-{:02}.jpg",
        date.year(),
        date.month(),
        date.day(),
        date.hour(),
        (date.minute() / 10) * 10
    );

    println!("Downloading image from: {}", url);
    let rep = get(&url).send().context("Failed to send request")?;

    if rep.status_code == 403 || rep.status_code == 404 {
        bail!("Image not available");
    }

    if rep.status_code != 200 {
        bail!("Failed to download image: HTTP {}", rep.status_code);
    }

    let body = rep.as_bytes();
    let path = std::env::current_dir().unwrap().join("image.jpg");
    write(&path, body).map_err(|e| anyhow!("Failed to write image to file: {}", e))?;
    println!("Image downloaded to: {}", path.display());

    Ok(())
}

fn main() {
    loop {
        let now_paris = Utc::now().with_timezone(&Paris);

        match update_wallpaper(now_paris) {
            Ok(_) => println!("Wallpaper updated successfully."),
            Err(e) => eprintln!("Error updating wallpaper: {}", e),
        }

        println!("Sleeping 8 minutes"); // Not ten, because if we are lagging behind we want to catch up
        sleep(Duration::from_secs(480));
    }
}
