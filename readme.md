# leptos_ssg

Powering [deadbaed](https://philippeloctaux.com/blog/)

## Features

- Content written in markdown
- HTML render of the content with my custom design
- Atom feed of the content
- Basic navigation: Previous / Next links on every article
- Works without JavaScript, it is used only to enhance content
- Generate custom views by inserting custom HTML tags in markdown source

## Tools used

- [leptos](https://leptos.dev) for HTML components
- [tailwindcss](http://tailwindcss.com) for CSS (reusing graphical design of https://philippeloctaux.com)
- [pulldown-cmark](https://crates.io/crates/pulldown-cmark) for markdown parsing

## TODO

- Add blurhash to images: [JavaScript](https://github.com/mad-gooze/fast-blurhash), [Rust](https://crates.io/crates/blurhash)
- Generate `sitemap.xml` for search indexers with [sitemap-rs](https://crates.io/crates/sitemap-rs)
- Tiny search engine to find content faster, instead of relying on an external tool
