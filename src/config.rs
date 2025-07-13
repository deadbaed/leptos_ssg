use jiff::Timestamp;

/// Configuration for a build
#[derive(Debug, Clone, Copy)]
pub struct BuildConfig<'a> {
    pub(crate) host: &'a str,
    pub(crate) base_url: &'a str,
    pub(crate) timestamp: Timestamp,
    pub(crate) stylesheet_name: &'a str,
    pub(crate) assets: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildConfigError<'a> {
    #[error("A trailing slash `/` is required at the end for `{0}`")]
    TrailingSlashRequired(&'a str),
    #[error("Failed to round timestamp to the nearest second")]
    RoundTimestampToSecond(jiff::Error),
}

impl<'a> BuildConfig<'a> {
    pub fn new(
        host: &'a str,
        base_url: &'a str,
        timestamp: Timestamp,
        stylesheet_name: &'a str,
        assets: &'a str,
    ) -> Result<Self, BuildConfigError<'a>> {
        if !base_url.ends_with("/") {
            return Err(BuildConfigError::TrailingSlashRequired(base_url));
        }

        let timestamp = timestamp
            .round(jiff::Unit::Second)
            .map_err(BuildConfigError::RoundTimestampToSecond)?;

        Ok(Self {
            host,
            base_url,
            timestamp,
            stylesheet_name,
            assets,
        })
    }
}
