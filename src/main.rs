fn main() {
    let target: std::path::PathBuf = "/Users/phil/x/leptos_ssg/target/blog".into();
    let config =
        leptos_ssg::BuildConfig::new("http://192.168.1.170:8000","/blog/", jiff::Timestamp::now(), "style.css", "./assets/")
            .unwrap();
    let content_path: std::path::PathBuf = "/Users/phil/x/blog/content/".into();
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
