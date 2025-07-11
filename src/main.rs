use leptos_ssg::html::prelude::*;

fn main() {
    let content_path = "/Users/phil/x/blog/content";

    let meta = leptos_ssg::html::BuildMeta::new("/", jiff::Timestamp::now()).unwrap();

    let content = leptos_ssg::Content::scan_path(content_path).unwrap();

    // index
    let posts_view = content
        .iter()
        .map(|post| {
            leptos::view! {
                <li class=tw_join!("flex", "flex-col", "lg:flex-wrap", "items-start")>
                    <a class=tw_join!("font-medium", "text-lg") href={format!("{}{}", meta.base_url, post.slug())} >{post.meta().title()}</a>
                    " "
                    <time datetime=post.meta().datetime().to_string() class=tw_join!("flex-none", "text-gray-400", "text-lg")>{post.meta().datetime().strftime("%F").to_string()}</time>
                </li>
            }
        })
        .collect::<Vec<_>>();

    let index_nav = leptos::view! {
        <nav>
            <ul class=tw_join!("flex", "flex-center", "my-2")>
                <li>{underline_link(format!("{}atom.xml", meta.base_url), "RSS", None)}</li>
                <span class=tw_join!("mx-2", "text-gray-400")>"-"</span>
                <li>{underline_link("https://philippeloctaux.com", "Website", None)}</li>
            </ul>
        </nav>
    };
    let index = leptos::view! {
        <ul class=tw_join!("space-y-6")>
            {posts_view}
        </ul>
    }
    .into_view();
    let index_page = leptos_ssg::html::blog(
        "deadbaed",
        "broke my bed, now it's dead",
        &meta,
        index_nav,
        index,
        (),
    )
    .to_html();
    std::fs::write("./target/site/index.html", index_page).unwrap();

    // Posts
    fn navigation_in_posts() -> impl IntoAny {
        leptos::view! {
            <nav>
                <ul class=tw_join!("flex", "flex-center", "my-2")>
                    <li>{underline_link("/", "← Home", None)}</li>
                </ul>
            </nav>
        }
    }
    for post in content {
        let publication_moment = format!("Posted on {} in {} ", post.meta().datetime().strftime("%B %d, %Y at %R"), post.meta().datetime().time_zone().iana_name().unwrap_or_default());
        let subtitle_view = view! {
            <div class=tw_join!("mt-4")>{publication_moment}<span data-relative-timestamp={post.meta().datetime().timestamp().as_millisecond()}></span></div>
        };
        let html = match post.generate_html() {
            Ok(html) => html,
            Err(e) => {
                println!("{e}");
                panic!();
            }
        };
        let view = view! {
          {leptos::html::article().inner_html(html)}
        }
        .into_any();
        let post_page = leptos_ssg::html::blog(
            post.meta().title(),
            subtitle_view,
            &meta,
            navigation_in_posts(),
            view,
            Some(leptos_ssg::html::syntax_highlight(post.code_block_languages())),
        )
        .to_html();
        let mut path: std::path::PathBuf = format!("./target/site/{}", post.slug()).into();
        std::fs::create_dir_all(&path).unwrap();
        path.push("index.html");
        std::fs::write(&path, post_page).unwrap();
        println!("wrote post {}", path.display());
    }

    // 404
    let notfound = leptos::view! {
        <div>"This page could not be found."</div>
        <div>"Perhaps the page you are looking for was moved, "{underline_link(meta.base_url.clone(), "go to the archive", None)}" to try finding it again?"</div>
    }
    .into_view();
    let not_found_page =
        leptos_ssg::html::content_page("404 Not Found", &meta, notfound, ()).to_html();
    std::fs::write("./target/site/404.html", not_found_page).unwrap();
}
