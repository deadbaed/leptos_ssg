use jiff::Zoned;
use std::{num::ParseIntError, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum SlugFromPostIdError {
    #[error("Could not find year in post id")]
    NoYear,
    #[error("Could not extract year from post id")]
    ConvertYear(ParseIntError),
    #[error("Year from post id mismatches with year from post metadata")]
    YearMismatchWithMetadata,

    #[error("Could not find month in post id")]
    NoMonth,
    #[error("Could not extract month from post id")]
    ConvertMonth(ParseIntError),
    #[error("Month from post id mismatches with month from post metadata")]
    MonthMismatchWithMetadata,

    #[error("Could not find day in post id")]
    NoDay,
    #[error("Could not extract day from post id")]
    ConvertDay(ParseIntError),
    #[error("Day from post id mismatches with day from post metadata")]
    DayMismatchWithMetadata,
}

pub(super) fn get_slug_from_post_id<S: AsRef<str>>(
    post_id: S,
    metadata_date_time: &Zoned,
) -> Result<String, SlugFromPostIdError> {
    // Split the string into parts using the hyphen as a delimiter
    let mut parts = post_id.as_ref().split('-');

    // The first part must be the year
    let year = parts.next().ok_or(SlugFromPostIdError::NoYear)?;
    let year = year
        .parse::<i16>()
        .map_err(SlugFromPostIdError::ConvertYear)?;
    year.eq(&metadata_date_time.year())
        .then_some(())
        .ok_or(SlugFromPostIdError::YearMismatchWithMetadata)?;

    // The second part must be the month
    let month = parts.next().ok_or(SlugFromPostIdError::NoMonth)?;
    let month = month
        .parse::<i8>()
        .map_err(SlugFromPostIdError::ConvertMonth)?;
    month
        .eq(&metadata_date_time.month())
        .then_some(())
        .ok_or(SlugFromPostIdError::MonthMismatchWithMetadata)?;

    // The third part must be the day
    let day = parts.next().ok_or(SlugFromPostIdError::NoDay)?;
    let day = day.parse::<i8>().map_err(SlugFromPostIdError::ConvertDay)?;
    day.eq(&metadata_date_time.day())
        .then_some(())
        .ok_or(SlugFromPostIdError::DayMismatchWithMetadata)?;

    // Combine the remaining parts into a single string
    let title = parts.collect::<Vec<&str>>().join("-");
    Ok(slug::slugify(title))
}

#[derive(Debug, thiserror::Error)]
pub enum GetPostIdError {
    #[error("Post has an invalid filename")]
    InvalidFilename,
    #[error("Parent directory of post has an invalid directory name")]
    InvalidParentDirectory,
}

pub(super) fn get_post_id(path: &Path) -> Result<&str, GetPostIdError> {
    match path.file_stem().and_then(|stem| stem.to_str()) {
        Some(file_stem) => match file_stem {
            "index" => {
                // Use parent folder name as post id
                Ok(path
                    .parent()
                    .and_then(|parent| parent.file_stem())
                    .and_then(|name| name.to_str())
                    .ok_or(GetPostIdError::InvalidParentDirectory)?)
            }
            file_stem => Ok(file_stem),
        },
        None => Err(GetPostIdError::InvalidFilename),
    }
}
