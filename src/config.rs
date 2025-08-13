use jiff::Timestamp;
use std::str::FromStr;
use uuid::Uuid;

/// Configuration for a build
#[derive(Debug, Clone, Copy)]
pub struct BuildConfig<'a> {
    pub(crate) host: &'a str,
    pub(crate) base_url: &'a str,
    pub(crate) timestamp: Timestamp,
    pub(crate) styles: Styles<'a>,
    pub(crate) assets: &'a str,
    pub(crate) logo: &'a str,
    pub(crate) website_name: &'a str,
    pub(crate) website_tagline: &'a str,
    pub(crate) content_author: &'a str,
    pub(crate) external_url: Option<&'a str>,
    pub(crate) feed_uuid: Uuid,
    #[cfg(feature = "opengraph")]
    pub(crate) webdriver: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct Styles<'a> {
    pub website: &'a str,
    #[cfg(feature = "opengraph")]
    pub opengraph: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildConfigError<'a> {
    #[error("A trailing slash `/` is required at the end for `{0}`")]
    TrailingSlashRequired(&'a str),
    #[error("Failed to round timestamp to the nearest second")]
    RoundTimestampToSecond(jiff::Error),
    #[error("Failed to parse provided timestamp: {0}")]
    ParseTimestamp(jiff::Error),
    #[error("Could not parse uuid: {0}")]
    Uuid(uuid::Error),
}

impl<'a> BuildConfig<'a> {
    pub fn new(
        host: &'a str,
        base_url: &'a str,
        timestamp: i64,
        styles: Styles<'a>,
        assets: &'a str,
        logo: &'a str,
        website_name: &'a str,
        website_tagline: &'a str,
        content_author: &'a str,
        external_url: Option<&'a str>,
        feed_uuid: &'a str,
        #[cfg(feature = "opengraph")] webdriver: &'a str,
    ) -> Result<Self, BuildConfigError<'a>> {
        if !base_url.ends_with("/") {
            return Err(BuildConfigError::TrailingSlashRequired(base_url));
        }

        let timestamp = Timestamp::from_second(timestamp)
            .map_err(BuildConfigError::ParseTimestamp)?
            .round(jiff::Unit::Second)
            .map_err(BuildConfigError::RoundTimestampToSecond)?;

        if !assets.ends_with("/") {
            return Err(BuildConfigError::TrailingSlashRequired(assets));
        }

        let feed_uuid = Uuid::from_str(feed_uuid).map_err(BuildConfigError::Uuid)?;

        Ok(Self {
            host,
            base_url,
            timestamp,
            styles,
            assets,
            logo,
            website_name,
            website_tagline,
            content_author,
            external_url,
            feed_uuid,
            #[cfg(feature = "opengraph")]
            webdriver,
        })
    }

    pub fn absolute_url(&self) -> String {
        format!("{}{}", self.host, self.base_url)
    }
}
