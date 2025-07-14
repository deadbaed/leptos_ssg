use jiff::Zoned;
use std::{num::ParseIntError, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum SlugFromContentIdError {
    #[error("Could not find year in content id")]
    NoYear,
    #[error("Could not extract year from content id")]
    ConvertYear(ParseIntError),
    #[error("Year from content id mismatches with year from content metadata")]
    YearMismatchWithMetadata,

    #[error("Could not find month in content id")]
    NoMonth,
    #[error("Could not extract month from content id")]
    ConvertMonth(ParseIntError),
    #[error("Month from content id mismatches with month from content metadata")]
    MonthMismatchWithMetadata,

    #[error("Could not find day in content id")]
    NoDay,
    #[error("Could not extract day from content id")]
    ConvertDay(ParseIntError),
    #[error("Day from content id mismatches with day from content metadata")]
    DayMismatchWithMetadata,
}

pub(super) fn get_slug_from_content_id<S: AsRef<str>>(
    content_id: S,
    metadata_date_time: &Zoned,
) -> Result<String, SlugFromContentIdError> {
    // Split the string into parts using the hyphen as a delimiter
    let mut parts = content_id.as_ref().split('-');

    // The first part must be the year
    let year = parts.next().ok_or(SlugFromContentIdError::NoYear)?;
    let year = year
        .parse::<i16>()
        .map_err(SlugFromContentIdError::ConvertYear)?;
    year.eq(&metadata_date_time.year())
        .then_some(())
        .ok_or(SlugFromContentIdError::YearMismatchWithMetadata)?;

    // The second part must be the month
    let month = parts.next().ok_or(SlugFromContentIdError::NoMonth)?;
    let month = month
        .parse::<i8>()
        .map_err(SlugFromContentIdError::ConvertMonth)?;
    month
        .eq(&metadata_date_time.month())
        .then_some(())
        .ok_or(SlugFromContentIdError::MonthMismatchWithMetadata)?;

    // The third part must be the day
    let day = parts.next().ok_or(SlugFromContentIdError::NoDay)?;
    let day = day
        .parse::<i8>()
        .map_err(SlugFromContentIdError::ConvertDay)?;
    day.eq(&metadata_date_time.day())
        .then_some(())
        .ok_or(SlugFromContentIdError::DayMismatchWithMetadata)?;

    // Combine the remaining parts into a single string
    let title = parts.collect::<Vec<&str>>().join("-");
    Ok(slug::slugify(title))
}

#[derive(Debug, thiserror::Error)]
pub enum GetContentIdError {
    #[error("Content has an invalid filename")]
    InvalidFilename,
    #[error("Parent directory of content has an invalid directory name")]
    InvalidParentDirectory,
}

pub(super) enum ContentId<'a> {
    Standalone(&'a str),
    WithAssets(&'a str),
}

impl<'a> AsRef<str> for ContentId<'a> {
    fn as_ref(&self) -> &str {
        match self {
            ContentId::Standalone(id) => id,
            ContentId::WithAssets(id) => id,
        }
    }
}

impl<'a> std::fmt::Display for ContentId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a> ContentId<'a> {
    pub fn from_path(path: &'a Path) -> Result<Self, GetContentIdError> {
        let file_stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or(GetContentIdError::InvalidFilename)?;

        match file_stem {
            "index" => {
                // Use parent folder name as content id
                path.parent()
                    .and_then(|parent| parent.file_stem())
                    .and_then(|name| name.to_str())
                    .ok_or(GetContentIdError::InvalidParentDirectory)
                    .map(Self::WithAssets)
            }
            file_stem => Ok(Self::Standalone(file_stem)),
        }
    }
}
