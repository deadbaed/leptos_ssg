fn main() {
    let target: std::path::PathBuf = "/Users/phil/x/leptos_ssg/target/aaaa".into();
    let target = target.as_path();

    let meta = leptos_ssg::html::BuildMeta::new("/", jiff::Timestamp::now()).unwrap();
    let content = leptos_ssg::content::Content::scan_path("/Users/phil/x/blog/content/").unwrap();

    let mut list_of_pages = vec![];

    list_of_pages.push(("404.html".into(), leptos_ssg::pages::not_found_page(meta)));
    list_of_pages.push((
        "index.html".into(),
        leptos_ssg::pages::index(&content, meta),
    ));

    let (ok, err): (Vec<_>, Vec<_>) = content
        .iter()
        .map(|content| (content.slug(), leptos_ssg::pages::content(content, meta)))
        .partition(|(_, html)| html.is_ok());

    let ok = ok
        .into_iter()
        .map(|(slug, html)| (format!("{slug}/index.html"), Result::unwrap(html)))
        .collect::<Vec<_>>();

    let err = err
        .into_iter()
        .map(|(slug, html)| (slug, Result::unwrap_err(html)))
        .collect::<Vec<_>>();

    if !err.is_empty() {
        println!("Failed to process the following pages:");
        for er in err {
            println!("{}: {}", er.0, er.1);
        }
    }

    ok.into_iter().for_each(|(slug, view)| {
        println!("Processed {slug}");
        list_of_pages.push((slug, view));
    });

    std::fs::create_dir_all(target).expect("target directory created");
    for (slug, view) in list_of_pages {
        let path = std::path::PathBuf::from(&slug);

        let parent = path.parent().expect("parent folder");
        std::fs::create_dir_all(target.join(parent))
            .expect("create parent folder of html document");

        let html_document = target.join(path);
        let html_document = html_document.as_path();
        match std::fs::write(html_document, leptos::prelude::RenderHtml::to_html(view)) {
            Ok(()) => println!("wrote `{slug}` to {}", html_document.display()),
            Err(e) => println!("failed to write `{slug}`: {}", e.kind()),
        }
    }
}
