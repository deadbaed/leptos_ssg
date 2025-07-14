use jiff::Zoned;
use pulldown_cmark::{Event, MetadataBlockKind, Tag, TagEnd};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Metadata {
    Title(String),
    Date(Zoned),
    Uuid(Uuid),
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataParseError {
    #[error("Could not find delimter in string `{0}`")]
    NoDelimeter(String),
    #[error("Tag `{0}` is unknown")]
    UnknownTag(String),
    #[error("Could not extract value out of tag `{0}`: {1}")]
    Value(String, ParseValueError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseValueError {
    #[error("Could not parse date/time: {0}")]
    DateTime(jiff::Error),
    #[error("Could not parse uuid: {0}")]
    Uuid(uuid::Error),
}

const TAG_TITLE: &str = "title";
const TAG_DATE: &str = "date";
const TAG_UUID: &str = "uuid";

impl FromStr for Metadata {
    type Err = MetadataParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        println!("Metadata: Parsing `{s}`");
        let (key, value) = s
            .split_once("=")
            .ok_or(MetadataParseError::NoDelimeter(s.into()))?;

        let key = key.trim();
        let value = value.trim();

        match key.to_lowercase().as_ref() {
            TAG_TITLE => {
                // Remove surrounding quotes in title
                let title = value.trim_matches('"');

                Ok(Self::Title(title.into()))
            }
            TAG_DATE => {
                let date = Zoned::from_str(value)
                    .map_err(ParseValueError::DateTime)
                    .map_err(|e| Self::Err::Value(key.into(), e))?;
                Ok(Self::Date(date))
            }
            TAG_UUID => {
                // Remove surrounding quotes in uuid
                let uuid = value.trim_matches('"');

                let uuid = Uuid::from_str(uuid)
                    .map_err(ParseValueError::Uuid)
                    .map_err(|e| Self::Err::Value(key.into(), e))?;
                Ok(Self::Uuid(uuid))
            }
            _ => Err(Self::Err::UnknownTag(key.into())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetadataList(Vec<Metadata>);

impl MetadataList {
    pub fn from_markdown(markdown_events: &[Event]) -> Result<Self, MetadataParseError> {
        let mut list = vec![];

        // Parse text only if we are inside the first metadata block
        let mut parse_text = false;

        for event in markdown_events {
            match event {
                Event::Text(text) => {
                    if parse_text {
                        for line in text.lines() {
                            list.push(Metadata::from_str(line)?);
                        }
                    }
                }
                Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle)) => {
                    break;
                }
                Event::Start(Tag::MetadataBlock(MetadataBlockKind::PlusesStyle)) => {
                    parse_text = true;
                }
                _ => {
                    // noop
                }
            }
        }

        // TODO: make sure that we have required elements, some like this:
        // fn count_variants<T, F>(&self, predicate: F) -> usize
        // where
        //     F: Fn(&Metadata) -> Option<T>,
        // {
        //     self.0.iter().filter(|el| predicate(el).is_some()).count()
        // }
        //
        // // How to use it
        // self.count_variants(|el| match el {
        //   Metadata::Title(_) => Some(()),
        //   _ => None,
        // }) == 1
        // // Wrap it inside a macro: with parameters: enum variant, number of occurences, return
        // Err if does not match

        Ok(Self(list))
    }

    pub fn title(&self) -> &str {
        self.0
            .iter()
            .find_map(|el| match el {
                Metadata::Title(title) => Some(title.as_str()),
                _ => None,
            })
            .expect("there should be a `Metadata::Title`")
    }

    pub fn datetime(&self) -> &Zoned {
        self.0
            .iter()
            .find_map(|el| match el {
                Metadata::Date(datetime) => Some(datetime),
                _ => None,
            })
            .expect("there should be a `Metadata::Date`")
    }

    pub fn uuid(&self) -> &Uuid {
        self.0
            .iter()
            .find_map(|el| match el {
                Metadata::Uuid(uuid) => Some(uuid),
                _ => None,
            })
            .expect("there should be a `Metadata::Uuid`")
    }
}
