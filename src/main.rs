use std::ffi::OsString;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicBool, AtomicU64};
use std::{fmt, io};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Local};
use futures::{stream, StreamExt};
use num_format::{Locale, ToFormattedString};

use crate::config::*;
use crate::feed::*;

mod config;
mod feed;

const LOC: &Locale = &Locale::en_GB;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()> {
    let now = Local::now();
    let config = AppConfig {
        reddit_domain: None,
        max_concurrent_downloads: None,
        out_path: Some(OsString::from("../reddit-feed-archive")),
        feeds: vec![
            FeedConfig {
                user_name: "<your-username-here>".to_string(),
                feed_token: "<your-feed-token-here>".to_string(),
                listings: Subset::All,
                formats: Subset::Some(vec![FeedFormat::Json]),
            },
            // ...feed configs for other accounts
        ],
    };

    let mut feeds = Vec::new();
    for feed_config in &config.feeds {
        for listing in &feed_config.listings.to_vec() {
            for format in &feed_config.formats.to_vec() {
                feeds.push(Feed::new(
                    feed_config.user_name.to_string(),
                    feed_config.feed_token.to_string(),
                    *listing,
                    *format,
                ));
            }
        }
    }

    download_feeds(
        &now,
        config.reddit_domain(),
        config.max_concurrent_downloads(),
        config.out_path(),
        &feeds,
    )
    .await?;
    Ok(())
}

async fn download_feeds(
    now: &DateTime<Local>,
    domain: &str,
    max_parallel_downloads: u16,
    out_dir: &Path,
    feeds: &[Feed],
) -> Result<(), ()> {
    let results = stream::iter(feeds)
        .map(|feed| async move { download_feed(now, domain, feed, out_dir).await })
        .buffer_unordered(max_parallel_downloads as usize);

    let total_bytes = AtomicU64::new(0);
    let succeeded = AtomicBool::new(true);
    results
        .for_each(|result| {
            let total_bytes = &total_bytes;
            let succeeded = &succeeded;
            async move {
                match result {
                    Ok((num_bytes, path)) => {
                        println!(
                            "Downloaded {} bytes to {}",
                            num_bytes.to_formatted_string(LOC),
                            path.to_string_lossy()
                        );
                        total_bytes.fetch_add(num_bytes, atomic::Ordering::Relaxed);
                    }
                    Err(err) => {
                        eprintln!("Failure! {}", err);
                        succeeded.store(false, atomic::Ordering::Relaxed);
                    }
                }
            }
        })
        .await;

    let total_bytes = total_bytes.into_inner();
    println!("Downloaded {} bytes", total_bytes.to_formatted_string(LOC));

    if succeeded.into_inner() {
        Ok(())
    } else {
        Err(())
    }
}

async fn download_feed(
    now: &DateTime<Local>,
    domain: &str,
    feed: &Feed,
    out_dir: &Path,
) -> Result<(u64, PathBuf), Error> {
    let file_path = out_dir.join(feed.sub_path(now));

    let content = feed.download(domain).await?;
    let num_bytes = write_bytes_to_file(&file_path, content)?;

    Ok((num_bytes, file_path))
}

fn write_bytes_to_file(path: &Path, content: Bytes) -> io::Result<u64> {
    if let Some(dir_path) = path.parent() {
        fs::create_dir_all(dir_path)?;
        let mut file = File::create(path)?;
        io::copy(&mut content.reader(), &mut file)
    } else {
        // path is the root of the file system!
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
}

#[derive(Debug)]
enum Error {
    Filesystem(io::Error),
    Network(reqwest::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Error::Filesystem(cause) => cause,
            Error::Network(cause) => cause,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::Filesystem(source)
    }
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Self::Network(source)
    }
}
