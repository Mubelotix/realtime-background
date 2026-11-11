use chrono_tz::Tz;
use minreq::get;
use std::fs::write;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{Context, bail, anyhow, Result as AnyResult};
use chrono::prelude::*;
use chrono_tz::Europe::Paris;

const REQUEST_TIMEOUT_SECONDS: u64 = 30;

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
    let rep = get(url)
        .with_timeout(REQUEST_TIMEOUT_SECONDS)
        .send()
        .context("Failed to send request")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Instant;

    fn assert_stalled_download_times_out(response: &'static [u8]) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/image.jpg", listener.local_addr().unwrap());
        let (release, released) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1];
            stream.read_exact(&mut request).unwrap();
            stream.write_all(response).unwrap();
            // Keep the connection open until the client times out. The extra
            // five seconds bound the test even if the client timeout regresses.
            let _ = released.recv_timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS + 5));
        });

        let start = Instant::now();
        let result = download_image(&url);
        let elapsed = start.elapsed();
        let _ = release.send(());
        server.join().unwrap();

        let error = result.unwrap_err();
        assert!(
            error.chain().any(|cause| {
                cause.downcast_ref::<std::io::Error>()
                    .is_some_and(|error| {
                        matches!(
                            error.kind(),
                            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                        )
                    })
            }),
            "expected a timeout, got: {error:#}"
        );
        assert!(elapsed < Duration::from_secs(REQUEST_TIMEOUT_SECONDS + 5));
    }

    #[test]
    fn times_out_waiting_for_response_headers() {
        assert_stalled_download_times_out(b"");
    }

    #[test]
    fn times_out_waiting_for_image_body() {
        assert_stalled_download_times_out(
            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial image",
        );
    }
}
