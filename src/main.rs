fn main() {
    let target: std::path::PathBuf = "/Users/phil/x/leptos_ssg/target/aaaa".into();
    let config = leptos_ssg::BuildConfig::new("/", jiff::Timestamp::now(), "style.css").unwrap();
    let mut blog = leptos_ssg::Blog::new(target, config);

    let content = leptos_ssg::Content::scan_path("/Users/phil/x/blog/content/").unwrap();
    blog.build_index_page(&content);
    blog.build_posts_pages(&content)
        .expect("processed markdown files");
    blog.build_404_page();

    let path = blog.write_files().expect("files written to disk");
    println!("Wrote files to {}", path.display());
}
