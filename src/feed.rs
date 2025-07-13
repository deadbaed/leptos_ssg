use atom_syndication::*;

fn jiff_to_chrono_date(zoned: &jiff::Zoned) -> chrono::DateTime<chrono::FixedOffset> {
    let rfc3339_date = zoned.strftime("%FT%T%:z").to_string();
    let chrono_date = chrono::DateTime::parse_from_rfc3339(&rfc3339_date).unwrap();

    chrono_date
}

pub fn create_feed(url: &str, content: &[crate::content::Content]) -> Feed {
    let mut feed = FeedBuilder::default();

    feed.id(crate::UUID);
    feed.lang(Some(crate::LANG.into()));
    feed.title(crate::TITLE);

    let subtitle = Text::plain(crate::SUBTITLE);
    feed.subtitle(subtitle);

    let mut link_atom = LinkBuilder::default();
    link_atom
        .href(format!("{url}atom.xml"))
        .rel("self")
        .mime_type(Some("application/atom+xml".into()));

    let mut link_html = LinkBuilder::default();
    link_html.href(url).mime_type(Some("text/html".into()));

    feed.links(vec![link_atom.build(), link_html.build()]);

    let mut generator = GeneratorBuilder::default();
    generator
        .uri(Some("https://github.com/deadbaed/leptos_ssg".into()))
        .value(env!("CARGO_PKG_NAME"))
        .version(Some(env!("CARGO_PKG_VERSION").into()));
    feed.generator(generator.build());

    let mut content_iter = content.iter().peekable();
    let most_recent_content = content_iter.peek();

    if let Some(content) = most_recent_content {
        feed.updated(jiff_to_chrono_date(content.meta().datetime()));
    }

    let mut author = PersonBuilder::default();
    let author = author
        .name("Philippe Loctaux")
        .uri(Some("https://philippeloctaux.com".into()))
        .build();

    let entries = content_iter
        .map(|content| {
            let mut entry = EntryBuilder::default();
            entry.author(author.clone());
            entry.title(content.meta().title());
            entry.published(jiff_to_chrono_date(content.meta().datetime()));
            entry.updated(jiff_to_chrono_date(content.meta().datetime()));

            // UUID is constructed with:
            // - blog UUID
            // - UUID of content: a UUID is required for every piece of content
            entry.id(format!(
                "urn:uuid:{}",
                uuid::Uuid::new_v5(crate::UUID.as_ref(), content.meta().uuid().as_ref())
                    .as_hyphenated()
                    .to_string()
            ));

            // URL
            let absolute_url = format!("{url}{}/", content.slug());
            let mut link_html = LinkBuilder::default();
            link_html
                .href(absolute_url)
                .mime_type(Some("text/html".into()));
            entry.link(link_html.build());

            // Content
            let mut content_feed = ContentBuilder::default();
            content_feed.lang(Some(crate::LANG.into()));
            content_feed.content_type(Some("html".into()));
            content_feed.value(content.raw_html(url));
            entry.content(Some(content_feed.build()));

            entry.build()
        })
        .collect::<Vec<_>>();

    feed.entries(entries);

    feed.build()
}
