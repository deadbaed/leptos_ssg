use std::time::SystemTime;

fn main() {
    let sys_time = SystemTime::now();
    let timestamp = sys_time
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("current timestamp")
        .as_secs()
        .try_into()
        .expect("timestamp in i64");

    #[cfg(debug_assertions)]
    let host = "http://localhost:4343";
    #[cfg(debug_assertions)]
    let base_url = "/example-site/";

    #[cfg(not(debug_assertions))]
    let host = "https://deadbaed.github.io";
    #[cfg(not(debug_assertions))]
    let base_url = "/leptos_ssg/";

    let assets = "./assets/".into();
    let target = "./target/example-site".into();
    let config = leptos_ssg::BuildConfig::new(
        host,
        base_url,
        timestamp,
        "style.css",
        assets,
        "leptos_circle.svg",
        "leptos_ssg",
        "simple site to showcase leptos_ssg",
        "John Doe",
        Some("https://github.com/deadbaed/leptos_ssg"),
        "00000000-0000-4000-0000-000000000000",
    )
    .unwrap();
    let content_path: std::path::PathBuf = "./content/".into();
    let mut blog = leptos_ssg::Blog::new(target, config);

    let content = leptos_ssg::Content::scan_path(&content_path).unwrap();
    blog.add_404_page();
    blog.add_index_page(&content);
    blog.add_content_pages(&content)
        .expect("processed markdown files");
    blog.add_content_assets(&content_path, &content);
    blog.add_atom_feed(&content);

    let path = blog.build().expect("files written to disk");
    println!("Wrote files to {}", path.display());
}
