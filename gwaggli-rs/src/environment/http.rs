use std::error::Error;
use std::{fs};
use std::cmp::min;
use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::get;
use url::Url;
use futures_util::StreamExt;

pub async fn download(src: Url, dest: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    if fs::metadata(&dest).is_ok() {
        return Err(format!("File {} already exists, unable to download.", dest.display()).into());
    }

    if fs::metadata(&dest.parent().unwrap()).is_err() {
        fs::create_dir_all(&dest.parent().unwrap())?;
    }

    println!("Downloading {}", src);

    let response = get(src.to_string()).await?;

    if response.status().is_success() {
        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) ")?
                .progress_chars("#>-"),
        );

        let mut file = File::create(&dest)?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message("Downloaded");
    } else {
        return Err(format!("HTTP Status {} while downloading {}", response.status(), &src).into());
    }

    Ok(dest)
}