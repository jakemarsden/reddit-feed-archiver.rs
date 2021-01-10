use std::ffi::OsString;

use bytes::Bytes;
use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct Feed {
    domain: String,
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

impl Feed {
    pub fn new(
        domain: String,
        user_name: String,
        token: String,
        listing: Listing,
        format: FeedFormat,
    ) -> Self {
        Self {
            domain,
            user_name,
            token,
            listing,
            format,
        }
    }

    pub async fn download(&self) -> reqwest::Result<Bytes> {
        let req_url = self.url();
        let res = reqwest::get(&req_url).await?.error_for_status()?;
        let res_body = res.bytes().await?;
        Ok(res_body)
    }

    pub fn url(&self) -> String {
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
            self.domain, url_path, ext, self.token, self.user_name
        )
    }

    pub fn file_name(&self, timestamp: &DateTime<Local>) -> OsString {
        let timestamp_str = timestamp.format("%Y-%m-%d_%H-%M-%S%z");
        let ext = self.format.extension();
        OsString::from(format!(
            "{}.{}.{}.{}",
            self.user_name,
            self.listing.file_name_part(),
            timestamp_str,
            ext
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
