mod content_id;
mod metadata;

use metadata::*;
use pulldown_cmark::Event;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use std::path::{Path, PathBuf};
use tailwind_fuse::tw_join;

#[derive(Debug, Clone)]
pub struct Content {
    path: PathBuf,
    raw: String,
    meta: MetadataList,
    slug: Slug,
    assets: Option<PathBuf>,

    // Navigation
    previous: Option<Slug>,
    next: Option<Slug>,
}

pub type Slug = String;

#[derive(Debug, thiserror::Error)]
pub enum ContentListError {
    #[error("Failed to read Content")]
    ReadFile(std::io::ErrorKind),
    #[error("Failed to parse metadata of content: {0}")]
    ParseMetadata(MetadataParseError),
    #[error("Failed to get ContentId: {0}")]
    ContentId(content_id::GetContentIdError),
    #[error("Failed to get ContentSlug: {0}")]
    ContentSlug(content_id::SlugFromContentIdError),
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
                let content_id =
                    content_id::ContentId::from_path(path).map_err(ContentListError::ContentId)?;
                let slug = content_id::get_slug_from_content_id(&content_id, meta.datetime())
                    .map_err(ContentListError::ContentSlug)?;

                // Assets
                let assets = match content_id {
                    content_id::ContentId::WithAssets(folder_name) => Some(folder_name.into()),
                    _ => None,
                };

                vec.push(Content {
                    path: path.to_path_buf(),
                    raw: file_contents,
                    meta,
                    slug,
                    assets,
                    previous: None,
                    next: None,
                });
            }
        }

        // Sort by descending order
        vec = Self::sort_desc(vec);

        // Add previous/next navigation
        {
            let mut previous = None;
            let mut iter = vec.iter_mut().peekable();

            while let Some(el) = iter.next() {
                el.previous = previous;
                previous = Some(el.slug());

                el.next = iter.peek().map(|el| el.slug());
            }
        }

        Ok(vec)
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

    pub fn assets(&self) -> Option<&Path> {
        self.assets.as_deref()
    }

    pub fn previous(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    pub fn next(&self) -> Option<&str> {
        self.next.as_deref()
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

    pub fn raw_html(&self, aboslute_url: &str) -> String {
        // Add instructions to read article on the website if formatting looks weird in feed reader
        let feed_bad_formatting_disclaimer = format!(
            "\n\n[If the formatting of this post looks odd in your feed reader, [visit the original article]({aboslute_url}{}/)]\n",
            self.slug()
        );
        let mut content_with_disclaimer = self.raw.clone();
        content_with_disclaimer.push_str(feed_bad_formatting_disclaimer.as_ref());

        // Parse markdown and write html
        let markdown_events = Self::markdown_events(content_with_disclaimer.as_ref());
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, markdown_events.into_iter());

        html_output
    }

    fn syntax_highlight_mapping(language: impl AsRef<str>) -> String {
        match language.as_ref() {
            "html" => "xml",
            language => language,
        }.into()
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
                    Some(Self::syntax_highlight_mapping(lang))
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GenerateHtmlError {
    #[error("unhandled markdown event: {0:?}")]
    UnknownMarkdownEvent(Event<'static>),
}

impl Content {
    pub fn generate_html(&self) -> Result<String, GenerateHtmlError> {
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

        // Image helper
        let mut inside_image = false;

        // List of views ready to be used
        let mut views = vec![];

        // Buffer when building a view, once finished it will be added to the list
        let mut current_view = String::new();

        for event in markdown_events {
            match (event, ignore) {
                // text
                (Event::Text(text), false) => {
                    if inside_image {
                        current_view.push_str(
                            format!(
                                "<blockquote class=\"{}\">{}</blockquote>",
                                tw_join!(
                                    "p-4",
                                    "mb-4",
                                    "border-l-8",
                                    "border-solid",
                                    "border-gray-500",
                                    "bg-gray-800"
                                ),
                                text.as_ref()
                            )
                            .as_ref(),
                        );
                    } else {
                        current_view.push_str(text.as_ref());
                    }
                }

                // markdown metadata
                (Event::Start(Tag::MetadataBlock(_)), _) => ignore = true,
                (Event::End(TagEnd::MetadataBlock(_)), _) => ignore = false,

                // titles
                (
                    Event::Start(Tag::Heading {
                        level,
                        id: _id,
                        classes: _classes,
                        attrs: _attrs,
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
                        link_type: _link_type,
                        dest_url,
                        title: _title,
                        id: _id,
                    }),
                    false,
                ) => {
                    inside_image = true;
                    current_view.push_str(
                        format!(
                            "<img loading=\"lazy\" src={dest_url} class=\"{}\" />",
                            tw_join!("my-4")
                        )
                        .as_ref(),
                    );
                }
                (Event::End(TagEnd::Image), _) => {
                    inside_image = false;
                }

                // links
                (
                    Event::Start(Tag::Link {
                        link_type: _link_type,
                        dest_url,
                        title: _title,
                        id: _id,
                    }),
                    false,
                ) => {
                    current_view.push_str(
                        format!(
                            "<a href=\"{dest_url}\" class=\"{}\">",
                            tw_join!("underline", "text-yellow-400", "break-all")
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
                            let language = Self::syntax_highlight_mapping(language);
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
                    current_view.push('\n');
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
                (Event::Start(Tag::BlockQuote(_kind)), false) => {
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
                (Event::End(TagEnd::BlockQuote(_kind)), false) => {
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

                // rule
                (Event::Rule, false) => {
                    current_view.push_str("<hr />");
                    views.push(current_view.clone());
                    current_view.clear();
                }

                // html blocks
                (Event::Start(Tag::HtmlBlock), false) => {} // noop
                (Event::End(TagEnd::HtmlBlock), false) => {} // noop

                // html
                (Event::Html(html), false) => {
                    let dom = match tl::parse(html.as_ref(), tl::ParserOptions::default()) {
                        Ok(dom) => dom,
                        Err(e) => {
                            println!("Failed to parse HTML: `{}`: {e}", html.as_ref());
                            continue;
                        }
                    };

                    struct CustomComponent {
                        tag: &'static str,
                        attribute: &'static str,
                        process: fn(&Content, &str) -> leptos::prelude::AnyView,
                    }

                    fn process_image_grid(
                        content: &Content,
                        attribute: &str,
                    ) -> leptos::prelude::AnyView {
                        use leptos::prelude::*;

                        let assets = if content.assets.is_some() {
                            // There are some assets, they are located in the parent directory
                            content.path.parent().unwrap()
                        } else {
                            // If there is no assets, then there is no html to render
                            return ().into_any();
                        };

                        let directory = assets.join(attribute);

                        // Collect list of images
                        let mut list_images = walkdir::WalkDir::new(&directory)
                            .into_iter()
                            .filter_map(|e| e.ok())
                            .map(|dir_entry| dir_entry.into_path())
                            .filter(|path| path.is_file())
                            // Get image files
                            .filter(|path| {
                                // TODO: allow svg files as well
                                infer::get_from_path(path)
                                    .map(|file_type| {
                                        file_type.is_some_and(|ft| {
                                            ft.matcher_type() == infer::MatcherType::Image
                                        })
                                    })
                                    .unwrap_or(false)
                            })
                            // Get relative path to be accepted in the html
                            .filter_map(|path| {
                                path.strip_prefix(assets)
                                    .map(|path| path.to_path_buf())
                                    .ok()
                            })
                            .collect::<Vec<_>>();

                        // Sort by name
                        list_images.sort();

                        // For each image, create html view
                        let list_images = list_images.into_iter().map(|path| {
                            let filename = path.file_name().and_then(|file| file.to_str()).map(|file| file.to_string()).unwrap();
                            let path = path.to_str().unwrap();

                            view! {
                                    <a class=tw_join!("w-full", "h-full", "border-2", "border-dashed", "border-yellow-600") href={path.to_string()}>
                                        <img loading="lazy" class=tw_join!("h-auto", "max-w-32") src={path.to_string()} alt=filename />
                                    </a>
                            }
                        }).collect_view();

                        // Final view with images
                        leptos::view! {
                            <div class=tw_join!("my-4", "grid", "grid-cols-2", "gap-5")>
                                {list_images}
                            </div>
                        }
                        .into_any()
                    }

                    const CUSTOM_COMPONENTS: &[CustomComponent] = &[CustomComponent {
                        tag: "ImageGrid",
                        attribute: "src",
                        process: process_image_grid,
                    }];

                    let mut view = None;

                    for custom_tag_name in CUSTOM_COMPONENTS {
                        let tag = match dom
                            .nodes()
                            .iter()
                            .find(|node| {
                                node.as_tag()
                                    .is_some_and(|tag| tag.name() == custom_tag_name.tag)
                            })
                            // Coerce as an html tag
                            .and_then(|node| node.as_tag())
                        {
                            Some(tag) => tag,
                            None => continue,
                        };

                        let attribute = match tag.attributes().get(custom_tag_name.attribute) {
                            Some(Some(attribute)) => attribute,
                            _ => continue,
                        };

                        let attribute = attribute.as_utf8_str();
                        view = Some((custom_tag_name.process)(self, attribute.as_ref()));
                    }

                    if let Some(view) = view {
                        current_view.push_str(leptos::prelude::RenderHtml::to_html(view).as_ref());
                        views.push(current_view.clone());
                        current_view.clear();
                    }
                }

                // ignored events
                (_, true) => {} // noop

                // unhandled events
                (event, false) => {
                    return Err(GenerateHtmlError::UnknownMarkdownEvent(event.into_static()));
                }
            }
        }

        let html = views.join("\n");
        Ok(html)
    }
}
