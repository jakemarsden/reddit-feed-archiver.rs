use std::{fmt, fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};

use bytes::{Buf, Bytes};
use chrono::prelude::*;
use num_format::{Locale, ToFormattedString};

use feed::*;

mod feed;

const REDDIT_DOMAIN: &str = "old.reddit.com";
const USER_NAME: &str = "<your-username-here>";
const FEED_TOKEN: &str = "<your-feed-token-here>";

const LOC: &Locale = &Locale::en_GB;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    download_feeds_to_file(&std::env::current_dir()?, REDDIT_DOMAIN, USER_NAME, FEED_TOKEN)
        .await?;
    Ok(())
}

async fn download_feeds_to_file(dir_path: &Path, domain: &str, user_name: &str, token: &str) -> Result<u64> {
    let listings = [
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
    let formats = [
        FeedFormat::Json,
        FeedFormat::Rss,
    ];

    let now = Local::now();

    let mut total_bytes = 0;
    for listing in &listings {
        let feed = Feed::new(domain.to_string(), user_name.to_string(), token.to_string(), *listing);

        for format in &formats {
            let (num_bytes, out_path) = download_feed(&feed, format, &dir_path, &now).await?;
            println!("Downloaded {} bytes to {}", num_bytes.to_formatted_string(LOC), out_path.to_string_lossy());
            total_bytes += num_bytes;
        }
    }

    println!("Downloaded total of {} bytes", total_bytes.to_formatted_string(LOC));
    Ok(total_bytes)
}

async fn download_feed(feed: &Feed, format: &FeedFormat, dir_path: &Path, now: &DateTime<Local>) -> Result<(u64, PathBuf)> {
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S%z");
    let file_name = format!("{}.{}.{}.{}", feed.user_name(), feed.listing().name(), timestamp, format.extension());
    let file_path = dir_path.join(file_name);

    let content = feed.download(format).await?;
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

type Result<TOk> = std::result::Result<TOk, Error>;

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
