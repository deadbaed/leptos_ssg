# leptos_ssg

A [static site generator](https://en.wikipedia.org/wiki/Static_site_generator) powered by Leptos.

View the [example demo site](https://deadbaed.github.io/leptos_ssg/)!

## Features

- Content written in markdown
- HTML render of the content with my custom design
- Atom feed of the content
- Basic navigation: Previous / Next links on every article
- Works without JavaScript, it is used only to enhance content
- Generate custom views by inserting custom HTML tags in markdown source

## Tools used

### Rust

I like it a lot

And of course because it's Blazing Fast ™

### pulldown-cmark

To parse content, [pulldown-cmark](https://crates.io/crates/pulldown-cmark) is used. It is used to parse content metadata, as well for the content itself.

It is used as-is for generating the Atom feed, but for the html render markdown events are handled manually to render some hand-crafted HTML.

### Leptos

[Leptos](https://leptos.dev) is used to create "templates" to be used for the website structure.

There is no webassembly here, I am simply using Leptos to output HTML.

I wanted to use it to render HTML from markdown, but I could not because of the markdown parser, so I had to craft HTML by hand. Otherwise, every time HTML needed to be created it was done through Leptos.

### TailwindCSS

I never liked writting CSS, but [TailwindCSS](http://tailwindcss.com) made me appreciate writting CSS.

Inside Leptos, every tailwind class was fed to [tailwind_fuse](https://crates.io/crates/tailwind_fuse) to join or handle conflicts.

## TODO

- Add blurhash to images: [JavaScript](https://github.com/mad-gooze/fast-blurhash), [Rust](https://crates.io/crates/blurhash)
- Generate `sitemap.xml` for search indexers with [sitemap-rs](https://crates.io/crates/sitemap-rs)
- Tiny search engine to find content faster, instead of relying on an external tool
- Tests
- More customizability
- Light/dark mode
- Respect gitignore when processing assets
- Add [tracing](https://crates.io/crates/tracing) for better logs
