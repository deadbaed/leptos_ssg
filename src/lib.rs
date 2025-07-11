mod config;
mod content;
mod html;
mod metadata;
mod pages;
mod post_id;

pub use config::BuildConfig;
pub use content::{Content, GenerateHtmlError};

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum BlogWriteFilesError {
    #[error("Content `{0}` does not have a parent folder")]
    NoParentFolder(PathBuf),

    #[error("Failed to created folder: {0}")]
    CreateFolder(std::io::ErrorKind),

    #[error("Failed to write to file {0}: {1}")]
    WriteFile(PathBuf, std::io::ErrorKind),
}

pub struct Blog<'a> {
    target: PathBuf,
    config: BuildConfig<'a>,
    pages: Vec<(PathBuf, leptos::prelude::AnyView)>,
}

#[cfg(debug_assertions)]
const EXTRA_FOLDER: &str = "";

#[cfg(debug_assertions)]
const WWW_FOLDER: &str = "";

#[cfg(not(debug_assertions))]
const EXTRA_FOLDER: &str = "extra/";

#[cfg(not(debug_assertions))]
const WWW_FOLDER: &str = "www/";

impl<'a> Blog<'a> {
    pub fn new(target: PathBuf, config: config::BuildConfig<'a>) -> Self {
        Self {
            target,
            config,
            pages: vec![],
        }
    }

    pub fn build_404_page(&mut self) {
        self.pages
            .push((format!("{EXTRA_FOLDER}404.html").into(), pages::not_found_page(self.config)));
    }

    pub fn build_index_page(&mut self, content: &[Content]) {
        self.pages
            .push((format!("{WWW_FOLDER}index.html").into(), pages::index(content, self.config)));
    }

    pub fn build_posts_pages(&mut self, content: &[Content]) -> Result<(), GenerateHtmlError> {
        let (ok, err): (Vec<_>, Vec<_>) = content
            .iter()
            .map(|content| (content.slug(), pages::content(content, self.config)))
            .partition(|(_, html)| html.is_ok());

        let ok = ok
            .into_iter()
            .map(|(slug, html)| (format!("{WWW_FOLDER}{slug}/index.html"), Result::unwrap(html)))
            .collect::<Vec<_>>();

        let err = err
            .into_iter()
            .map(|(slug, html)| (slug, Result::unwrap_err(html)))
            .collect::<Vec<_>>();

        if !err.is_empty() {
            println!("Failed to process the following pages:");
            for er in err {
                println!("{}: {}", er.0, er.1);
            }
        }

        ok.into_iter().for_each(|(slug, view)| {
            println!("Processed {slug}");
            self.pages.push((slug.into(), view));
        });

        Ok(())
    }

    pub fn write_files(self) -> Result<PathBuf, BlogWriteFilesError> {
        let target = self.target.as_path();

        for (slug, view) in self.pages {
            let path = PathBuf::from(&slug);

            // Create all parent directories
            let parent = path
                .parent()
                .ok_or(BlogWriteFilesError::NoParentFolder(slug.clone()))?;
            std::fs::create_dir_all(target.join(parent))
                .map_err(|e| BlogWriteFilesError::CreateFolder(e.kind()))?;

            // Write html to file
            let html = leptos::prelude::RenderHtml::to_html(view);
            let html_bytes = if cfg!(debug_assertions) {
                html.into_bytes()
            } else {
                // Minify everything
                let mut cfg = minify_html::Cfg::new();
                cfg.minify_js = true;

                minify_html::minify(html.as_ref(), &cfg)
            };

            let html_document = target.join(path);
            std::fs::write(&html_document, html_bytes)
                .map_err(|e| BlogWriteFilesError::WriteFile(html_document.clone(), e.kind()))?;
            println!("wrote `{}` to {}", slug.display(), html_document.display());
        }

        Ok(self.target)
    }
}
