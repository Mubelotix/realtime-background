use chrono_tz::Tz;
use minreq::get;
use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{Context, bail, anyhow, Result as AnyResult};
use chrono::prelude::*;
use chrono_tz::Europe::Paris;

fn get_url(date: DateTime<Tz>) -> String {
    format!(
        "https://skaping.s3.gra.io.cloud.ovh.net/amboise/la-pagode/snap/{}/{:02}/{:02}/{:02}-{:02}.jpg",
        date.year(),
        date.month(),
        date.day(),
        date.hour(),
        (date.minute() / 10) * 10
    )
}

fn download_image(url: &str) -> AnyResult<()> {
    println!("Downloading image from: {}", url);
    let rep = get(url).send().context("Failed to send request")?;

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
    let mut latest_success_url = String::new();
    loop {
        let mut current = Utc::now().with_timezone(&Paris);

        loop {
            let current_url = get_url(current);
            if current_url <= latest_success_url {
                println!("Image already available");
                break;
            }

            match download_image(&current_url) {
                Ok(_) => {
                    println!("Image updated successfully!");
                    latest_success_url = current_url;
                    break;
                },
                Err(e) if e.to_string() == "Image not available" => {
                    println!("Image not yet available. Trying previous image.");
                    current -= chrono::Duration::minutes(10);
                    sleep(Duration::from_secs(5));
                    continue;
                }
                Err(e) => {
                    println!("Error updating image: {e}. Aborting this cycle.");
                    break;
                },
            }
        }

        println!("Sleeping 60 seconds");
        sleep(Duration::from_secs(60));
    }
}
