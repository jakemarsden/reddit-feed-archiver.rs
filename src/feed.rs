use std::ffi::OsString;
use std::path::PathBuf;

use bytes::Bytes;
use chrono::{DateTime, Local};

use crate::config::AllValues;

#[derive(Clone)]
pub struct Feed {
    user_name: String,
    token: String,
    listing: Listing,
    format: FeedFormat,
}

#[derive(Copy, Clone)]
pub enum FeedFormat {
    Json,
    Rss,
}

impl AllValues for FeedFormat {
    fn all() -> &'static [Self] {
        static ALL: &[FeedFormat] = &[FeedFormat::Json, FeedFormat::Rss];
        ALL
    }
}

#[derive(Copy, Clone)]
pub enum Listing {
    FrontPage,
    Saved,

    UpVoted,
    DownVoted,
    Hidden,

    InboxAll,
    InboxUnread,
    InboxMessages,
    InboxCommentReplies,
    InboxSelfPostReplies,
    InboxMentions,
}

impl AllValues for Listing {
    fn all() -> &'static [Self] {
        static ALL: &[Listing] = &[
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
        ALL
    }
}

impl Feed {
    pub fn new(user_name: String, token: String, listing: Listing, format: FeedFormat) -> Self {
        Self {
            user_name,
            token,
            listing,
            format,
        }
    }

    pub async fn download(&self, domain: &str) -> reqwest::Result<Bytes> {
        let req_url = self.url(domain);
        let res = reqwest::get(&req_url).await?.error_for_status()?;
        let res_body = res.bytes().await?;
        Ok(res_body)
    }

    pub fn url(&self, domain: &str) -> String {
        let url_path: String = match &self.listing {
            Listing::FrontPage => "/".to_string(),
            Listing::Saved => "/saved".to_string(),

            Listing::UpVoted => format!("/user/{}/upvoted", self.user_name),
            Listing::DownVoted => format!("/user/{}/downvoted", self.user_name),
            Listing::Hidden => format!("/user/{}/hidden", self.user_name),

            Listing::InboxAll => "/message/inbox/".to_string(),
            Listing::InboxUnread => "/message/unread/".to_string(),
            Listing::InboxMessages => "/message/messages/".to_string(),
            Listing::InboxCommentReplies => "/message/comments/".to_string(),
            Listing::InboxSelfPostReplies => "/message/selfreply".to_string(),
            Listing::InboxMentions => "/message/mentions".to_string(),
        };
        let ext = self.format.extension();
        format!(
            "https://{}{}.{}?feed={}&user={}",
            domain, url_path, ext, self.token, self.user_name
        )
    }

    pub fn sub_path(&self, timestamp: &DateTime<Local>) -> PathBuf {
        PathBuf::from(&self.user_name)
            .join(timestamp.format("%Y-%m-%d_%H-%M-%S%z").to_string())
            .join(self.file_name())
    }

    pub fn file_name(&self) -> OsString {
        OsString::from(format!(
            "{}.{}",
            self.listing.file_name_part(),
            self.format.extension()
        ))
    }
}

impl FeedFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Rss => "rss",
        }
    }
}

impl Listing {
    fn file_name_part(&self) -> &str {
        match self {
            Self::FrontPage => "frontpage",
            Self::Saved => "saved",

            Self::UpVoted => "upvoted",
            Self::DownVoted => "downvoted",
            Self::Hidden => "hidden",

            Self::InboxAll => "inbox",
            Self::InboxUnread => "inbox_unread",
            Self::InboxMessages => "inbox_messages",
            Self::InboxCommentReplies => "inbox_commentReplies",
            Self::InboxSelfPostReplies => "inbox_selfPostReplies",
            Self::InboxMentions => "inbox_mentions",
        }
    }
}
