pub mod prelude {
    pub use super::icons::*;
    pub use super::underline_link;
    pub use leptos::prelude::*;
    pub use tailwind_fuse::tw_join;
    pub use tailwind_fuse::tw_merge;
}
mod icons;

use crate::config::BuildConfig;
use jiff::Timestamp;
use prelude::*;

pub fn syntax_highlight(languages: impl Iterator<Item = impl AsRef<str>>) -> impl IntoView {
    const HIGHLIGHT_JS_URL: &str = "https://unpkg.com/@highlightjs/cdn-assets@11.11.1";

    // Load core highlight.js
    let load_js = format!("import hljs from \"{HIGHLIGHT_JS_URL}/es/core.min.js\";");

    // Make sure every language appears only once
    let languages = languages
        .map(|language| language.as_ref().to_string())
        .collect::<std::collections::HashSet<_>>();

    // Load js file for every language, and register it to highlight.js
    let languages = languages.into_iter().map(|l| {
        format!("import {l} from \"{HIGHLIGHT_JS_URL}/es/languages/{l}.min.js\"; hljs.registerLanguage(\"{l}\", {l});")
    }).collect::<Vec<_>>().join("\n");

    // Highlight code blocks, and remove placeholder CSS while highlight.js was loading
    let highlight_blocks = r#"
document.addEventListener("DOMContentLoaded", (event) => {
  document.querySelectorAll("pre code").forEach((el) => hljs.highlightElement(el));
  document.querySelectorAll("pre").forEach((el) => {
    el.removeAttribute("class");

    // Add padding for scrollbar
    el.classList.add("pb-4");
  });
});"#;

    let syntax_highlight = [load_js, languages, highlight_blocks.into()].join("\n");

    view! {
        <link rel="stylesheet" href=format!("{HIGHLIGHT_JS_URL}/styles/default.min.css") />
        <script type="module" inner_html=syntax_highlight></script>
    }
}

pub fn underline_link(
    url: impl ToString,
    children: impl IntoAny,
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let class = tw_merge!("underline", "text-yellow-400", class);
    view! {
        <a href=url.to_string() class=class>
            {children.into_any()}
        </a>
    }
    .into_view()
}

pub fn navigation(children: impl IntoAny) -> impl IntoView {
    view! {
        <nav>
            <ul class=tw_join!("flex", "flex-center", "my-2", "space-x-6")>
                {children.into_any()}
            </ul>
        </nav>
    }
}

fn footer(timestamp: &Timestamp) -> impl IntoView {
    view! {
        <footer class=tw_join!("bg-black")>
            <div class=tw_join!("container", "mx-auto", "py-8", "px-4", "sm:px-8", "md:px-16", "lg:px-32", "xl:px-64", "2xl:px-96")>
                <p>{format!("Page generated on {timestamp} ")}<span data-relative-timestamp={timestamp.as_millisecond()}></span></p>
            </div>
        </footer>
    }
}

fn stats() -> impl IntoView {
    view! {
        <script inner_html=r#"
            window.goatcounter = {
                path: function(p) { return location.host + p }
            };
        "#></script>
        <script data-goatcounter="https://goatcounter.philt3r.eu/count" async src="https://goatcounter.philt3r.eu/count.js"></script>
    }
}

pub fn shell(
    title: &str,
    config: BuildConfig,
    children: impl IntoAny,
    additional_js: impl IntoAny,
) -> AnyView {
    const SUFFIX: &str = "deadbaed";
    let title = if title != SUFFIX {
        format!("{title} - {SUFFIX}")
    } else {
        title.into()
    };

    let relative_timestamp = r#"
function formatRelativeTime(durationInSeconds) {
  const units = [
    { unit: "year", seconds: 31536000 },
    { unit: "month", seconds: 2592000 },
    { unit: "week", seconds: 604800 },
    { unit: "day", seconds: 86400 },
    { unit: "hour", seconds: 3600 },
    { unit: "minute", seconds: 60 },
    { unit: "second", seconds: 1 }
  ];

  const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  // Find the best unit to use
  for (const { unit, seconds } of units) {
    if (Math.abs(durationInSeconds) >= seconds) {
      const value = durationInSeconds / seconds;
      return rtf.format(Math.round(value), unit);
    }
  }

  // Fallback in case the duration is zero
  return "now";
}

const currentDateTime = new Date().getTime();
const elements = document.querySelectorAll("[data-relative-timestamp]");

elements.forEach(element => {
  const timestamp = element.getAttribute("data-relative-timestamp");
  const diffInMilliseconds = new Date(+timestamp) - currentDateTime;
  const diffInSeconds = Math.floor(diffInMilliseconds / 1000);

  const relativeTime = formatRelativeTime(diffInSeconds);

  element.textContent = `(${relativeTime})`;
});"#;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width" />
                <link rel="stylesheet" href={format!("{}{}", config.base_url, config.stylesheet_name)} />
                <title>{title}</title>
            </head>

            <body class=tw_join!("flex", "flex-col", "min-h-screen", "bg-gray-900", "text-white")>
                <main class=tw_join!("flex-grow")>
                    {children.into_any()}
                </main>
                {footer(&config.timestamp)}
                <script inner_html=relative_timestamp></script>
                {stats()}
                {additional_js.into_any()}
            </body>
        </html>
    }
    .into_any()
}

fn container(children: impl IntoAny) -> impl IntoAny {
    view! {
        <div class=tw_join!("container", "mx-auto", "py-16", "px-4", "sm:px-8", "md:px-16", "lg:px-32", "xl:px-64", "2xl:px-96")>
            {children.into_any()}
        </div>
    }
}

fn title_view(title: &str, subtitle: impl IntoAny) -> AnyView {
    view! {
        <div>
            <h1 class=tw_join!("text-4xl", "font-bold")>{title.to_string()}</h1>
            <div class=tw_join!("text-xl", "font-medium")>{subtitle.into_any()}</div>
        </div>
    }
    .into_any()
}

fn nav_and_content(header: impl IntoAny, children: impl IntoAny) -> AnyView {
    view! {
        <div class=tw_join!("mt-8")>
            {header.into_any()}
            <hr />
        </div>
        <div class=tw_join!("my-4")>{children.into_any()}</div>
    }
    .into_any()
}

pub fn blog(
    title: &str,
    subtitle: impl IntoAny,
    config: BuildConfig,
    header: impl IntoAny,
    children: impl IntoAny,
    additional_js: impl IntoAny,
) -> AnyView {
    shell(
        title,
        config,
        container(view! {
            {title_view(title, subtitle).into_any()}
            {nav_and_content(header, children).into_any()}
        }),
        additional_js,
    )
    .into_any()
}

pub fn home(
    title: &str,
    subtitle: impl IntoAny,
    config: BuildConfig,
    header: impl IntoAny,
    children: impl IntoAny,
    additional_js: impl IntoAny,
) -> AnyView {
    shell(
        title,
        config,
        container(view! {
            <div class=tw_join!("md:flex", "md:space-x-4")>
                <div class=tw_join!("mb-2", "md:mb-0")>
                    <img src=format!("{}philt3r.png", config.base_url) class=tw_join!("w-42", "h-42", "rounded-2xl", "hover:rounded-full", "hover:motion-safe:animate-spin", "hover:cursor-wait") />
                </div>
                {title_view(title, subtitle).into_any()}
            </div>
            {nav_and_content(header, children).into_any()}
        }),
        additional_js,
    )
    .into_any()
}
