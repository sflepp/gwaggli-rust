use crate::environment::fs::{download_cache_dir, prepare_download_cache_dir};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use rand::distributions::{Alphanumeric, DistString};
use reqwest::get;
use std::cmp::min;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

pub async fn download(src: Url, dest: PathBuf) -> Result<PathBuf, Box<dyn Error>> {
    if fs::metadata(&dest).is_ok() {
        return Err(format!(
            "File {} already exists, unable to download.",
            dest.display()
        )
        .into());
    }

    if fs::metadata(dest.parent().unwrap()).is_err() {
        fs::create_dir_all(dest.parent().unwrap())?;
    }

    prepare_download_cache_dir();

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

        let tmp_file = temp_download_file();
        let mut file = File::create(&tmp_file)?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        fs::rename(tmp_file, dest.clone())?;

        pb.finish_with_message("Downloaded");
    } else {
        return Err(format!(
            "HTTP Status {} while downloading {}",
            response.status(),
            &src
        )
        .into());
    }

    Ok(dest)
}

fn temp_download_file() -> PathBuf {
    let filename = Alphanumeric.sample_string(&mut rand::thread_rng(), 10);
    download_cache_dir().join(filename)
}
