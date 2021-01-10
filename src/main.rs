use std::{fmt, io};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{self, AtomicBool, AtomicU64};

use bytes::{Buf, Bytes};
use chrono::{DateTime, Local};
use futures::{stream, StreamExt};
use num_format::{Locale, ToFormattedString};

use feed::*;

mod feed;

const REDDIT_DOMAIN: &str = "old.reddit.com";
const USER_NAME: &str = "<your-username-here>";
const FEED_TOKEN: &str = "<your-feed-token-here>";

const MAX_PARALLEL_DOWNLOADS: usize = 32;
const LOC: &Locale = &Locale::en_GB;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ()> {
    let curr_dir = &std::env::current_dir()
        .map_err(|err| eprintln!("Failed to get current directory: {}", err))?;
    download_feeds(curr_dir, REDDIT_DOMAIN, USER_NAME, FEED_TOKEN).await?;
    Ok(())
}

async fn download_feeds(dir_path: &Path, domain: &str, user_name: &str, token: &str) -> Result<(), ()> {
    static LISTINGS: &[Listing] = &[
        Listing::FrontPage,
        Listing::Saved,
        Listing::UpVoted,
        Listing::DownVoted,
        Listing::Hidden,
        Listing::InboxAll,
        Listing::InboxUnread,
        Listing::InboxMessages,
        Listing::InboxCommentReplies,
        Listing::InboxSelfPostReplies,
        Listing::InboxMentions,
    ];
    static FORMATS: &[FeedFormat] = &[
        FeedFormat::Json,
        FeedFormat::Rss,
    ];

    let now = Local::now();
    let feeds: Vec<Feed> = {
        let mut out = Vec::with_capacity(LISTINGS.len() * FORMATS.len());
        for listing in LISTINGS {
            for format in FORMATS {
                out.push(
                    Feed::new(domain.to_string(), user_name.to_string(), token.to_string(), *listing, *format));
            }
        }
        out
    };

    let results = stream::iter(feeds)
        .map(|feed| async move {
            download_feed(&feed, &dir_path, &now).await
        })
        .buffer_unordered(MAX_PARALLEL_DOWNLOADS);

    let total_bytes = AtomicU64::new(0);
    let succeeded = AtomicBool::new(true);
    results
        .for_each(|result| {
            let total_bytes = &total_bytes;
            let succeeded = &succeeded;
            async move {
                match result {
                    Ok((num_bytes, path)) => {
                        println!("Downloaded {} bytes to {}", num_bytes.to_formatted_string(LOC), path.to_string_lossy());
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
    let succeeded = succeeded.into_inner();

    println!("Downloaded {} bytes", total_bytes.to_formatted_string(LOC));
    if succeeded { Ok(()) } else { Err(()) }
}

async fn download_feed(feed: &Feed, dir_path: &Path, now: &DateTime<Local>) -> Result<(u64, PathBuf), Error> {
    let file_path = dir_path.join(feed.file_name(now));

    let content = feed.download().await?;
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
