use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::feed::{FeedFormat, Listing};

const DEFAULT_MAX_CONCURRENT_DOWNLOADS: u16 = 32;
const DEFAULT_REDDIT_DOMAIN: &str = "old.reddit.com";
const DEFAULT_OUT_PATH: &str = ""; // current dir

#[derive(Clone)]
pub struct AppConfig {
    pub reddit_domain: Option<String>,
    pub max_concurrent_downloads: Option<u16>,
    pub out_path: Option<OsString>,
    pub feeds: Vec<FeedConfig>,
}

#[derive(Clone)]
pub struct FeedConfig {
    pub user_name: String,
    pub feed_token: String,
    pub listings: Subset<Listing>,
    pub formats: Subset<FeedFormat>,
}

#[derive(Clone)]
pub enum Subset<T> {
    All,
    Some(Vec<T>),
}

pub trait AllValues
where
    Self: Sized,
{
    fn all() -> &'static [Self];
}

impl AppConfig {
    pub fn reddit_domain(&self) -> &str {
        match &self.reddit_domain {
            Some(domain) => domain,
            None => DEFAULT_REDDIT_DOMAIN,
        }
    }

    pub fn max_concurrent_downloads(&self) -> u16 {
        self.max_concurrent_downloads
            .unwrap_or(DEFAULT_MAX_CONCURRENT_DOWNLOADS)
    }

    pub fn out_path(&self) -> &Path {
        let path = match &self.out_path {
            Some(path) => path.as_os_str(),
            None => OsStr::new(DEFAULT_OUT_PATH),
        };
        Path::new(path)
    }
}

impl<T: 'static + AllValues + Clone> Subset<T> {
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            Self::All => T::all().to_vec(),
            Self::Some(them) => them.clone(),
        }
    }
}
