mod config;
mod content;
mod feed;
mod html;
mod pages;

const TITLE: &str = "deadbaed";
const SUBTITLE: &str = "broke my bed, now it's dead";
const LANG: &str = "en";

pub use config::BuildConfig;
pub use content::{Content, GenerateHtmlError};

use atom_syndication::Feed;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum BlogWriteFilesError {
    #[error("File `{0}` does not have a parent folder")]
    NoParentFolder(PathBuf),

    #[error("Failed to created folder: {0}")]
    CreateFolder(std::io::ErrorKind),

    #[error("Failed to write to file {0}: {1}")]
    WriteFile(PathBuf, std::io::ErrorKind),
}

#[derive(Debug)]
struct CopyAsset {
    source: PathBuf,
    target: PathBuf,
}

pub struct Blog<'config> {
    target: PathBuf,
    config: BuildConfig<'config>,
    pages: Vec<(PathBuf, leptos::prelude::AnyView)>,
    assets: Vec<CopyAsset>,
    atom_feed: Option<Feed>,
}

#[cfg(debug_assertions)]
const EXTRA_FOLDER: &str = "";

#[cfg(debug_assertions)]
const WWW_FOLDER: &str = "";

#[cfg(not(debug_assertions))]
const EXTRA_FOLDER: &str = "extra/";

#[cfg(not(debug_assertions))]
const WWW_FOLDER: &str = "www/";

impl<'config> Blog<'config> {
    pub fn new(target: PathBuf, config: config::BuildConfig<'config>) -> Self {
        Self {
            target,
            config,
            pages: vec![],
            assets: vec![],
            atom_feed: None,
        }
    }

    pub fn add_404_page(&mut self) {
        self.pages.push((
            format!("{EXTRA_FOLDER}404.html").into(),
            pages::not_found_page(self.config),
        ));
    }

    pub fn add_index_page(&mut self, content: &[Content]) {
        self.pages.push((
            format!("{WWW_FOLDER}index.html").into(),
            pages::index(content, self.config),
        ));
    }

    pub fn add_content_pages(&mut self, content: &[Content]) -> Result<(), GenerateHtmlError> {
        let (ok, err): (Vec<_>, Vec<_>) = content
            .iter()
            .map(|content| (content.slug(), pages::content(content, self.config)))
            .partition(|(_, html)| html.is_ok());

        let ok = ok
            .into_iter()
            .map(|(slug, html)| {
                (
                    format!("{WWW_FOLDER}{slug}/index.html"),
                    Result::unwrap(html),
                )
            })
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

    fn add_assets(assets: &mut Vec<CopyAsset>, source_base: &Path, target_base: &Path) {
        // Gather list of source assets
        let source_assets = walkdir::WalkDir::new(&source_base)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|dir_entry| dir_entry.into_path())
            .filter(|path| {
                infer::get_from_path(path)
                    .map(|file_type| {
                        file_type.map_or(false, |ft| {
                            use infer::MatcherType;
                            ft.matcher_type() == MatcherType::Image
                                || ft.matcher_type() == MatcherType::Audio
                                || ft.matcher_type() == MatcherType::Video
                        })
                    })
                    .unwrap_or(false)
            });

        // For each source asset, get its target path
        let source_and_target_assets = source_assets.filter_map(|source| {
            // Remove source prefix
            source
                .strip_prefix(&source_base)
                .map(|path| path.to_path_buf())
                // Add target prefix instead
                .map(|path| target_base.join(path))
                .ok()
                // Keep both source and target paths
                .map(|target| (source, target))
        });

        // Add to the list of assets
        source_and_target_assets.for_each(|(source, target)| {
            assets.push(CopyAsset { source, target });
        });
    }

    pub fn add_content_assets(&mut self, content_path: &Path, content: &[Content]) {
        content
            .iter()
            // The names are not the same between content and final folder (since it is public)
            .flat_map(|content| content.assets().map(|assets| (assets, content.slug())))
            .for_each(|(assets, slug)| {
                // Base paths for source and target locations
                let source_base = content_path.join(assets);
                let target_base = self.target.join(slug);

                Self::add_assets(&mut self.assets, &source_base, &target_base);
            });
    }

    pub fn add_atom_feed(&mut self, content: &[Content]) {
        let absolute_url = format!("{}{}", self.config.host, self.config.base_url);
        self.atom_feed = Some(feed::create_feed(absolute_url.as_ref(), content));
    }

    pub fn write_view_to_file(
        view: leptos::prelude::AnyView,
        base_path: &Path,
        path: &Path,
    ) -> Result<PathBuf, BlogWriteFilesError> {
        // Create all parent directories
        let parent = path
            .parent()
            .ok_or(BlogWriteFilesError::NoParentFolder(path.to_path_buf()))?;
        std::fs::create_dir_all(base_path.join(parent))
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

        let html_document = base_path.join(&path);
        std::fs::write(&html_document, html_bytes)
            .map_err(|e| BlogWriteFilesError::WriteFile(html_document.clone(), e.kind()))?;
        println!("wrote `{}` to {}", path.display(), html_document.display());

        Ok(html_document)
    }

    pub fn copy_asset(source: &Path, target: &Path) -> Result<(), BlogWriteFilesError> {
        // Create parent directory if it does note exist
        let parent = target
            .parent()
            .ok_or(BlogWriteFilesError::NoParentFolder(target.to_path_buf()))?;
        std::fs::create_dir_all(parent).map_err(|e| BlogWriteFilesError::CreateFolder(e.kind()))?;

        // Copy file
        std::fs::copy(source, target)
            .map_err(|e| BlogWriteFilesError::WriteFile(target.to_path_buf(), e.kind()))?;

        println!("copied `{}` to `{}`", source.display(), target.display());

        Ok(())
    }

    fn write_atom_feed(
        atom_feed: Feed,
        target: &Path,
    ) -> Result<PathBuf, BlogWriteFilesError> {
        let path = target.join("atom.xml");
        std::fs::write(&path, atom_feed.to_string())
            .map_err(|e| BlogWriteFilesError::WriteFile(path.clone(), e.kind()))?;

        Ok(path)
    }

    /// Consume struct, write to files
    pub fn build(mut self) -> Result<PathBuf, BlogWriteFilesError> {
        // Add internal assets
        let assets_path = PathBuf::from(self.config.assets);
        Self::add_assets(
            &mut self.assets,
            assets_path.as_path(),
            self.target.as_path(),
        );

        for (path, view) in self.pages {
            Self::write_view_to_file(view, self.target.as_path(), &path)?;
        }

        for copy_asset in self.assets {
            Self::copy_asset(&copy_asset.source, &copy_asset.target)?;
        }

        if let Some(atom_feed) = self.atom_feed {
            Self::write_atom_feed(atom_feed, &self.target)?;
        }

        Ok(self.target)
    }
}
