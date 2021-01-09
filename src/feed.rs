use bytes::Bytes;

pub type Error = reqwest::Error;
pub type Result<TOk> = std::result::Result<TOk, Error>;

pub struct Feed {
    domain: String,
    user_name: String,
    token: String,
    listing: Listing,
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
    pub fn new(domain: String, user_name: String, token: String, listing: Listing) -> Self {
        Self { domain, user_name, token, listing }
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    pub fn listing(&self) -> &Listing {
        &self.listing
    }

    pub async fn download(&self, format: &FeedFormat) -> Result<Bytes> {
        let req_url = self.url(format);
        let res = reqwest::get(&req_url)
            .await?
            .error_for_status()?;
        let res_body = res.bytes().await?;
        Ok(res_body)
    }

    pub fn url(&self, format: &FeedFormat) -> String {
        let ext = format.extension();
        let url_path = match &self.listing {
            Listing::FrontPage => format!("/.{}", ext),
            Listing::Saved => format!("/saved.{}", ext),

            Listing::UpVoted => format!("/user/{}/upvoted.{}", self.user_name, ext),
            Listing::DownVoted => format!("/user/{}/downvoted.{}", self.user_name, ext),
            Listing::Hidden => format!("/user/{}/hidden.{}", self.user_name, ext),

            Listing::InboxAll => format!("/message/inbox/.{}", ext),
            Listing::InboxUnread => format!("/message/unread/.{}", ext),
            Listing::InboxMessages => format!("/message/messages/.{}", ext),
            Listing::InboxCommentReplies => format!("/message/comments/.{}", ext),
            Listing::InboxSelfPostReplies => format!("/message/selfreply.{}", ext),
            Listing::InboxMentions => format!("/message/mentions.{}", ext),
        };
        format!("https://{}{}?feed={}&user={}", self.domain, url_path, self.token, self.user_name)
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
    pub fn name(&self) -> &str {
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
