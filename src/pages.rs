use crate::config::BuildConfig;
use crate::content::{Content, GenerateHtmlError};
use crate::html::prelude::*;

pub fn not_found_page<'a>(config: BuildConfig<'a>) -> AnyView {
    let view = leptos::view! {
        <div>"This page could not be found."</div>
        <div>"Perhaps the page you are looking for was moved, "{underline_link(config.base_url, "go to the archive", None)}" to find it!"</div>
    }
    .into_view();

    crate::html::blog(
        "404 Not Found",
        icon_face_frown(None),
        config,
        crate::html::navigation(view! {
            <li>{underline_link("/",view!{ {icon_home(None)}"Home" }, None)}</li>
        }),
        view,
        (),
    )
}

pub fn content(content: &Content, config: BuildConfig) -> Result<AnyView, GenerateHtmlError> {
    let subtitle = view! {
            <div class=tw_join!("mt-4")>{format!(
            "Posted on {} in {} ",
            content.meta().datetime().strftime("%B %d, %Y at %R"),
            content.meta()
                .datetime()
                .time_zone()
                .iana_name()
                .unwrap_or_default()
        )}<span data-relative-timestamp={content.meta().datetime().timestamp().as_millisecond()}></span></div>
    };

    let content_html = content.generate_html()?;

    Ok(crate::html::blog(
        content.meta().title(),
        subtitle,
        config,
        crate::html::navigation(view! {
            <li>{underline_link("/",view!{ {icon_home(None)}"Home" }, None)}</li>
        }),
        leptos::html::article().inner_html(content_html),
        Some(crate::html::syntax_highlight(
            content.code_block_languages(),
        )),
    ))
}

pub fn index<'a>(content: &[Content], config: BuildConfig<'a>) -> AnyView {
    let view = content
        .iter()
        .map(|post| {
            leptos::view! {
                <li class=tw_join!("flex", "flex-col", "lg:flex-wrap", "items-start")>
                    <a class=tw_join!("font-medium", "text-lg") href={format!("{}{}", config.base_url, post.slug())} >{post.meta().title()}</a>
                    " "
                    <time datetime=post.meta().datetime().to_string() class=tw_join!("flex-none", "text-gray-400", "text-lg")>{post.meta().datetime().strftime("%F").to_string()}</time>
                </li>
            }
        }).collect_view();

    crate::html::blog(
        "deadbaed",
        "broke my bed, now it's dead",
        config,
        crate::html::navigation(view! {
                <li>{underline_link(format!("{}atom.xml", config.base_url), view!{ {icon_rss(Some(tw_join!("text-yellow-400")))}"RSS" }, None)}</li>
                <li>{underline_link("https://philippeloctaux.com", view!{ {icon_website(Some(tw_join!("text-yellow-400")))}"Website" }, None)}</li>
        }),
        view! {
            <ul class=tw_join!("space-y-6")>
                {view}
            </ul>
        },
        (),
    )
}
