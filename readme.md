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

### TailwindCSS

I never liked writting CSS, but [TailwindCSS](http://tailwindcss.com) made me appreciate writting CSS.

Inside Leptos, every tailwind class was fed to [tailwind_fuse](https://crates.io/crates/tailwind_fuse) to join or handle conflicts.

## Quirks

### Leptos to generate HTML from markdown

I initially wanted to use Leptos to render markdown events as HTML components. Unfortunately, the Leptos macro requires a complete HTML tag to work:
```rust
let my_view = leptos::prelude::view! {
    <p> // The macro will not accept this
    <p></p> // This is okay
};
```

The markdown parser emits markdown events, a lot of them are broken down into start and end events, for exemple with paragraphs:
```rust
match markdown_events {
    Event::Start(Tag::Paragraph) => {
        // Add "<p>"
    }
    Event::End(TagEnd::Paragraph) => {
        // Add "</p>"
    }
    Event::Text(text) => {
        // This event will be fired anytime the markdown contains raw text
    }
    _ => {},
}
```

Those two behaviors require to manully write HTML tags by hand, which is fine, I just wanted to use Leptos everywhere I could!

Maybe there is another way to handle that. If you have ideas how I would love to know that!

## TODO

- Add blurhash to images: [JavaScript](https://github.com/mad-gooze/fast-blurhash), [Rust](https://crates.io/crates/blurhash)
- Generate `sitemap.xml` for search indexers with [sitemap-rs](https://crates.io/crates/sitemap-rs)
- Tiny search engine to find content faster, instead of relying on an external tool
- Tests
- More customizability
- Respect gitignore when processing assets
- Add [tracing](https://crates.io/crates/tracing) for better logs
- Be [HTML compliant](https://validator.w3.org/nu/?doc=https%3A%2F%2Fdeadbaed.github.io%2Fleptos_ssg%2F)

## License

- Code is licensed under the MIT license
- Image assets located in the ./example directory are copyrighted by their respective authors.
