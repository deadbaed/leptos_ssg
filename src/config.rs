use jiff::Timestamp;

/// Configuration for a build
#[derive(Debug, Clone, Copy)]
pub struct BuildConfig<'a> {
    pub(crate) base_url: &'a str,
    pub(crate) timestamp: Timestamp,
    pub(crate) stylesheet_name: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildConfigError {
    #[error("A trailing slash `/` is required at the end of the base url")]
    BaseUrlRequiresTrailingSlash,
    #[error("Failed to round timestamp to the nearest second")]
    RoundTimestampToSecond(jiff::Error),
}

impl<'a> BuildConfig<'a> {
    pub fn new(
        base_url: &'a str,
        timestamp: Timestamp,
        stylesheet_name: &'a str,
    ) -> Result<Self, BuildConfigError> {
        if !base_url.ends_with("/") {
            return Err(BuildConfigError::BaseUrlRequiresTrailingSlash);
        }

        let timestamp = timestamp
            .round(jiff::Unit::Second)
            .map_err(BuildConfigError::RoundTimestampToSecond)?;

        Ok(Self {
            base_url,
            timestamp,
            stylesheet_name,
        })
    }
}
