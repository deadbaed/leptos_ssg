use crate::config::BuildConfig;
use crate::content::{Content, GenerateHtmlError};
use crate::html::prelude::*;

pub fn not_found_page<'a>(
    config: BuildConfig<'a>,
    additional_js: Option<impl leptos::prelude::IntoAny>,
) -> AnyView {
    let view = leptos::view! {
        <div>"This page could not be found."</div>
        <div>"Perhaps the page you are looking for was moved, "{underline_link(config.base_url, "go to the homepage", None)}" to find it!"</div>
    }
    .into_view();

    crate::html::blog(
        true,
        "404 Not Found",
        config.website_name,
        icon_face_frown(None),
        config,
        crate::html::navigation(view! {
            <li>{underline_link(config.base_url, view!{ {icon_home(None)}"Home" }, None)}</li>
        }),
        view,
        additional_js
            .map(|js| js.into_any())
            .unwrap_or(().into_any()),
        ().into_any(),
    )
}

pub fn content(
    content: &Content,
    config: BuildConfig,
    additional_js: Option<impl leptos::prelude::IntoAny>,
) -> Result<AnyView, GenerateHtmlError> {
    println!("Processing content `{}`", content.slug());

    let subtitle = view! {
            <div class=tw_join!("mt-4")>{format!(
            "Posted on {} ",
            content.meta().datetime().strftime("%B %d, %Y at %R"),
        )}<span data-relative-timestamp={content.meta().datetime().timestamp().as_millisecond()}></span></div>
    };

    let content_html = content.generate_html()?;

    // Calling `content.next()` because the list is sorted in descending order
    let previous_navigation = content.next().map(|slug| {
        view! {
            <li>{underline_link(format!("{}{slug}", config.base_url),view!{ {icon_arrow_uturn_left(None)}"Previous" }, None)}</li>
        }.into_any()
    }).unwrap_or(().into_any());

    // Calling `content.previous()` because the list is sorted in descending order
    let next_navigation = content.previous().map(|slug| {
        view! {
            <li>{underline_link(format!("{}{slug}", config.base_url),view!{ {icon_arrow_uturn_right(None)}"Next" }, None)}</li>
        }.into_any()
    }).unwrap_or(().into_any());

    // Additional JS
    let additional_js = view! {
        {crate::html::syntax_highlight(content.code_block_languages()).into_any()}
        {additional_js.map(|js| js.into_any()).unwrap_or(().into_any())}
    };

    let url = format!("{}{}", config.absolute_url(), content.slug());
    let additional_meta = view! {
        <link rel="canonical" href=url.clone() />
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:type", "article").into_any()
        }
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:image", format!("{url}/opengraph.png")).into_any()
        }
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:url", url).into_any()
        }
    };

    Ok(crate::html::blog(
        false,
        content.meta().title(),
        config.website_name,
        subtitle,
        config,
        crate::html::navigation(view! {
            <li>{underline_link(config.base_url, view!{ {icon_home(None)}"Home" }, None)}</li>
            {previous_navigation}
            {next_navigation}
        }),
        leptos::html::article().inner_html(content_html),
        Some(additional_js),
        Some(additional_meta),
    ))
}

pub fn index<'a>(
    content: &[Content],
    config: BuildConfig<'a>,
    additional_js: Option<impl leptos::prelude::IntoAny>,
) -> AnyView {
    let view = content
        .iter()
        .map(|content| {
            leptos::view! {
                <li class=tw_join!("flex", "flex-col", "lg:flex-wrap", "items-start")>
                    <a class=tw_join!("font-medium", "text-lg") href={format!("{}{}", config.base_url, content.slug())} >{content.meta().title()}</a>
                    " "
                    <time datetime=content.meta().datetime().strftime(crate::RFC_3339_FORMAT).to_string() class=tw_join!("flex-none", "text-gray-600", "dark:text-gray-400", "text-lg")>{content.meta().datetime().strftime("%F").to_string()}</time>
                </li>
            }
        }).collect_view();

    let external_website = config
        .external_url
        .map(|url| {
            view! {
                <li>{underline_link(url, view!{ {icon_website(None)}"Website" }, None)}</li>
            }
            .into_any()
        })
        .unwrap_or(().into_any());

    let additional_meta = view! {
        <link rel="canonical" href=config.absolute_url() />
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:description", config.website_tagline).into_any()
        }
        <meta name="description" content=config.website_tagline />
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:type", "website").into_any()
        }
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:image", format!("{}opengraph.png", config.absolute_url())).into_any()
        }
        {
            #[cfg(feature = "opengraph")]
            add_opengraph_property("og:url", config.absolute_url()).into_any()
        }
    };

    crate::html::home(
        config.website_name,
        config.website_name,
        config.website_tagline,
        config,
        crate::html::navigation(view! {
                <li>{underline_link(format!("{}atom.xml", config.base_url), view!{ {icon_rss(None)}"Web feed" }, None)}</li>
                {external_website}
        }),
        view! {
            <ul class=tw_join!("space-y-6") data-testid="content-list" >
                {view}
            </ul>
        },
        additional_js
            .map(|js| js.into_any())
            .unwrap_or(().into_any()),
        additional_meta,
    )
}
