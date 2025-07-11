use crate::metadata::*;
use crate::post_id;
use pulldown_cmark::Event;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use std::path::Path;
use tailwind_fuse::tw_join;

#[derive(Debug, Clone)]
pub struct Content {
    raw: String,
    meta: MetadataList,
    slug: Slug,
}

type Slug = String;

#[derive(Debug, thiserror::Error)]
pub enum ContentListError {
    #[error("Failed to read content of Post")]
    ReadFile(std::io::ErrorKind),
    #[error("Failed to parse metadata of Post: {0}")]
    ParseMetadata(MetadataParseError),
    #[error("Failed to get Post Id: {0}")]
    PostId(post_id::GetPostIdError),
    #[error("Failed to get Post Slug: {0}")]
    PostSlug(post_id::SlugFromPostIdError),
}

impl Content {
    pub fn scan_path<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, ContentListError> {
        let mut vec = Vec::new();

        for entry in walkdir::WalkDir::new(path.as_ref())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|x| x == "md") {
                let file_contents = std::fs::read_to_string(path)
                    .map_err(|e| ContentListError::ReadFile(e.kind()))?;

                // Parse markdown metadata blocks
                let events = Self::markdown_events(&file_contents);
                let meta = MetadataList::from_markdown(&events)
                    .map_err(ContentListError::ParseMetadata)?;

                // Get slug out of filename
                let post_id = post_id::get_post_id(path).map_err(ContentListError::PostId)?;
                let slug = post_id::get_slug_from_post_id(post_id, &meta.datetime())
                    .map_err(ContentListError::PostSlug)?;

                vec.push(Content {
                    raw: file_contents,
                    meta,
                    slug,
                });
            }
        }

        // Sort by descending order
        Ok(Self::sort_desc(vec))
    }

    /// Sort by descending order, the first item is the newest one
    fn sort_desc(vec: Vec<Self>) -> Vec<Self> {
        let mut vec = vec;
        vec.sort_by(|a, b| a.meta.datetime().cmp(b.meta.datetime()).reverse());
        vec
    }

    pub fn slug(&self) -> Slug {
        self.slug.clone()
    }

    pub fn meta(&self) -> &MetadataList {
        &self.meta
    }

    fn markdown_events<'input>(input: &'input str) -> Vec<Event<'input>> {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        let parser = pulldown_cmark::Parser::new_ext(input, options);

        let iterator = pulldown_cmark::TextMergeStream::new(parser);
        iterator.collect()
    }

    /// Collect languages found in code blocks in markdown content
    pub fn code_block_languages(&self) -> impl Iterator<Item = impl AsRef<str>> {
        Self::markdown_events(&self.raw)
            .into_iter()
            .filter_map(|event| {
                if let Event::Start(Tag::CodeBlock(kind)) = event
                    && let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind
                    && lang != pulldown_cmark::CowStr::Borrowed("")
                {
                    Some(lang)
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateHtmlError<'a> {
    #[error("unhandled markdown event: {0:?}")]
    UnknownMarkdownEvent(Event<'a>),
}

impl<'a> Content {
    pub fn generate_html(&'a self) -> Result<String, GenerateHtmlError<'a>> {
        let markdown_events = Content::markdown_events(&self.raw);
        let mut ignore = false;

        // table helpers
        enum TableState {
            Head,
            Body,
        }
        let mut table_alignment = vec![];
        let mut table_cell_idx = 0;
        let mut table_state = TableState::Head;

        use leptos::prelude::*;

        // List of views ready to be used
        let mut views = vec![];

        // Buffer when building a view, once finished it will be added to the list
        let mut current_view = String::new();

        for event in markdown_events {
            match (event, ignore) {
                // text
                (Event::Text(text), false) => current_view.push_str(text.as_ref()),

                // markdown metadata
                (Event::Start(Tag::MetadataBlock(_)), _) => ignore = true,
                (Event::End(TagEnd::MetadataBlock(_)), _) => ignore = false,

                // titles
                (
                    Event::Start(Tag::Heading {
                        level,
                        id,
                        classes,
                        attrs,
                    }),
                    false,
                ) => {
                    current_view.push_str(
                        format!(
                            "<{level} class=\"{}\">",
                            match level {
                                pulldown_cmark::HeadingLevel::H1 =>
                                    tw_join!("my-6", "font-bold", "text-4xl"),
                                pulldown_cmark::HeadingLevel::H2 =>
                                    tw_join!("my-6", "font-bold", "text-3xl"),
                                pulldown_cmark::HeadingLevel::H3 =>
                                    tw_join!("my-6", "font-bold", "text-2xl"),
                                pulldown_cmark::HeadingLevel::H4 =>
                                    tw_join!("my-6", "font-bold", "text-xl"),
                                pulldown_cmark::HeadingLevel::H5 =>
                                    tw_join!("my-6", "font-bold", "text-lg"),
                                pulldown_cmark::HeadingLevel::H6 =>
                                    tw_join!("my-6", "font-bold", "text-md"),
                            }
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::Heading(level)), false) => {
                    current_view.push_str(format!("</{level}>").as_ref());
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // paragraph
                (Event::Start(Tag::Paragraph), false) => current_view.push_str(
                    format!("<p class=\"{}\">", tw_join!("my-1.5", "text-justify")).as_ref(),
                ),
                (Event::End(TagEnd::Paragraph), false) => {
                    current_view.push_str("</p>\n");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // images
                (
                    Event::Start(Tag::Image {
                        link_type,
                        dest_url,
                        title,
                        id,
                    }),
                    false,
                ) => {
                    current_view.push_str(
                        format!("<img loading=\"lazy\" src={dest_url} alt=\"{title}\" />").as_ref(),
                    );
                }
                (Event::End(TagEnd::Image), _) => {} // noop

                // links
                (
                    Event::Start(Tag::Link {
                        link_type,
                        dest_url,
                        title,
                        id,
                    }),
                    false,
                ) => {
                    current_view.push_str(
                        format!(
                            "<a href=\"{dest_url}\" class=\"{}\">",
                            tw_join!("underline", "text-yellow-400")
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::Link), _) => {
                    current_view.push_str("</a>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // lists
                (Event::Start(Tag::List(first_items)), false) => {
                    let ordered = first_items.is_some();
                    let tag = match ordered {
                        true => format!(
                            "<ol class=\"{}\">",
                            tw_join!("ml-4", "pl-4", "list-decimal")
                        ),
                        false => {
                            format!("<ul class=\"{}\">", tw_join!("ml-4", "pl-4", "list-disc"))
                        }
                    };
                    current_view.push_str(tag.as_ref());
                }
                (Event::End(TagEnd::List(ordered)), false) => {
                    let tag = match ordered {
                        true => "</ol>",
                        false => "</ul>",
                    };
                    current_view.push_str(tag.as_ref());
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // list items
                (Event::Start(Tag::Item), false) => {
                    current_view.push_str("<li>");
                }
                (Event::End(TagEnd::Item), false) => {
                    current_view.push_str("</li>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // italic
                (Event::Start(Tag::Emphasis), false) => {
                    current_view
                        .push_str(format!("<em class=\"{}\">", tw_join!("italic")).as_ref());
                }
                (Event::End(TagEnd::Emphasis), false) => {
                    current_view.push_str("</em>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // bold
                (Event::Start(Tag::Strong), false) => {
                    current_view
                        .push_str(format!("<strong class=\"{}\">", tw_join!("font-bold")).as_ref());
                }
                (Event::End(TagEnd::Strong), false) => {
                    current_view.push_str("</strong>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // code inline
                (Event::Code(code), false) => {
                    current_view.push_str(
                        format!(
                            "<code class=\"{}\">{}</code>",
                            tw_join!("font-mono", "bg-white", "text-black", "px-1", "py-0.5"),
                            code.as_ref(),
                        )
                        .as_ref(),
                    );
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // line break
                (Event::HardBreak, false) => {
                    current_view.push_str("<br />");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // code block
                (Event::Start(Tag::CodeBlock(code_block)), false) => {
                    let class_code_language = match code_block {
                        pulldown_cmark::CodeBlockKind::Indented
                        | pulldown_cmark::CodeBlockKind::Fenced(
                            pulldown_cmark::CowStr::Borrowed(""),
                        ) => "".into(),
                        pulldown_cmark::CodeBlockKind::Fenced(language) => {
                            format!(" class=\"language-{language}\"")
                        }
                    };
                    current_view.push_str(
                        format!(
                            "<pre class=\"{}\"><code{}>",
                            tw_join!(
                                "overflow-x-scroll",
                                "font-mono",
                                "bg-white",
                                "text-black",
                                "p-4",
                            ),
                            class_code_language
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::CodeBlock), false) => {
                    current_view.push_str("</code></pre>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // softbreak
                (Event::SoftBreak, false) => {
                    current_view.push_str("\n");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // table
                (Event::Start(Tag::Table(alignment)), false) => {
                    table_alignment = alignment;

                    current_view.push_str(
                        format!("<div class=\"{}\">", tw_join!("overflow-x-auto", "my-4")).as_ref(),
                    );

                    current_view.push_str(
                        format!(
                            "<div class=\"{}\">",
                            tw_join!("inline-block", "min-w-full", "align-middle",)
                        )
                        .as_ref(),
                    );

                    current_view.push_str(
                        format!(
                            "<table class=\"{}\">",
                            tw_join!("min-w-full", "divide-y", "divide-gray-700")
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::Table), false) => {
                    current_view.push_str("</tbody></table>");
                    current_view.push_str("</div></div>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // table head
                (Event::Start(Tag::TableHead), false) => {
                    table_state = TableState::Head;
                    table_cell_idx = 0;
                    current_view.push_str("<thead><tr>");
                }
                (Event::End(TagEnd::TableHead), false) => {
                    current_view.push_str("</tr></thead>");
                    current_view.push_str(
                        format!(
                            "<tbody class=\"{}\">",
                            tw_join!("divide-y", "divide-gray-800")
                        )
                        .as_ref(),
                    );
                    views.push(current_view.clone());
                    current_view.clear();

                    table_state = TableState::Body;
                }

                // table row
                (Event::Start(Tag::TableRow), false) => {
                    table_cell_idx = 0;
                    current_view.push_str("<tr>");
                }
                (Event::End(TagEnd::TableRow), false) => {
                    current_view.push_str("</tr>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // table cell
                (Event::Start(Tag::TableCell), false) => {
                    let class_table_state = match table_state {
                        TableState::Head => {
                            current_view.push_str("<th");
                            tw_join!(
                                "px-3",
                                "py-3.5",
                                "text-left",
                                "text-sm",
                                "font-semibold",
                                "text-white"
                            )
                        }
                        TableState::Body => {
                            current_view.push_str("<td");
                            tw_join!(
                                "px-3",
                                "py-4",
                                "text-sm",
                                "whitespace-nowrap",
                                "text-gray-300"
                            )
                        }
                    };
                    let class_table_alignment = match table_alignment.get(table_cell_idx) {
                        Some(&pulldown_cmark::Alignment::Left) => {
                            tw_join!("text-left")
                        }
                        Some(&pulldown_cmark::Alignment::Center) => {
                            tw_join!("text-center")
                        }
                        Some(&pulldown_cmark::Alignment::Right) => {
                            tw_join!("text-right")
                        }
                        _ => tw_join!(""),
                    };

                    current_view.push_str(
                        format!(
                            " class=\"{}\">",
                            tw_join!(class_table_state, class_table_alignment)
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::TableCell), false) => {
                    match table_state {
                        TableState::Head => {
                            current_view.push_str("</th>");
                        }
                        TableState::Body => {
                            current_view.push_str("</td>");
                        }
                    }
                    table_cell_idx += 1;

                    views.push(current_view.clone());
                    current_view.clear();
                }

                // quotes
                (Event::Start(Tag::BlockQuote(kind)), false) => {
                    current_view.push_str(
                        format!(
                            "<blockquote class=\"{}\">",
                            tw_join!(
                                "p-4",
                                "my-4",
                                "border-l-8",
                                "border-solid",
                                "border-gray-500",
                                "bg-gray-800"
                            )
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::BlockQuote(kind)), false) => {
                    current_view.push_str("</blockquote>");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // checkboxes
                (Event::TaskListMarker(checked), false) => {
                    let checked = if checked { "checked" } else { "" };
                    current_view.push_str(
                        format!(
                            "<input type=\"checkbox\" {checked} class=\"{}\" />",
                            tw_join!("accent-yellow-600")
                        )
                        .as_ref(),
                    );
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // html blocks
                (Event::Start(Tag::HtmlBlock), false) => {} // noop
                (Event::End(TagEnd::HtmlBlock), false) => {} // noop

                // html
                (Event::Html(html), false) => {
                    println!("raw html `{html}`");
                }

                // ignored events
                (_, true) => {} // noop

                // unhandled events
                (event, false) => return Err(GenerateHtmlError::UnknownMarkdownEvent(event)),
            }
        }

        let html = views.join("\n");
        Ok(html)
    }
}
