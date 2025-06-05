use minreq::get;
use std::fs::write;
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

fn set_gnome_background(path: PathBuf) -> Result<(), String> {
    let uri = format!("file://{}", path.canonicalize().map_err(|e| e.to_string())?.display());

    // Commands to run
    let commands = [
        ("org.gnome.desktop.background", "picture-uri", &uri),
        ("org.gnome.desktop.background", "picture-uri-dark", &uri),
    ];

    for (schema, key, value) in commands.iter() {
        let status = Command::new("gsettings")
            .args(["set", schema, key, value])
            .status()
            .map_err(|e| format!("Failed to run gsettings: {}", e))?;

        if !status.success() {
            return Err(format!(
                "gsettings set failed for {} {} {}",
                schema, key, value
            ));
        }
    }

    Ok(())
}


fn set_wallpaper(path: PathBuf) {
    if is_gsettings_available() {
        set_gnome_background(path).expect("Failed to set GNOME background");
    } else {
        wallpaper::set_from_path(path.to_str().unwrap()).expect("Failed to set wallpaper");
    }
}

fn main() {
    let url = "https://data.skaping.com/amboise-quais-de-loire/photo/2025/06/05/22-10.jpg";
    let req = get(url).send().unwrap();
    if req.status_code != 200 {
        panic!("Failed to download image: {}", req.status_code);
    }
    let body = req.as_bytes();
    write("image.jpg", body).expect("Unable to write file");

    let full_path = std::env::current_dir().unwrap().join("image.jpg");
    println!("Image downloaded to: {}", full_path.display());

    set_wallpaper(full_path.clone());

}
