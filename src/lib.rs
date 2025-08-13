mod config;
mod content;
mod feed;
mod html;
mod pages;

const LANG: &str = "en";
const RFC_3339_FORMAT: &str = "%FT%T%:z";

pub use config::{BuildConfig, Styles};
pub use content::{Content, GenerateHtmlError};

use atom_syndication::Feed;
use leptos::prelude::{AnyView, RenderHtml};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum BlogWriteFilesError {
    #[error("File `{0}` does not have a parent folder")]
    NoParentFolder(PathBuf),

    #[error("Failed to created folder: {0}")]
    CreateFolder(std::io::ErrorKind),

    #[error("Failed to write to file {0}: {1}")]
    WriteFile(PathBuf, std::io::ErrorKind),

    #[error("Failed to copy file to {0}: {1}")]
    CopyFile(PathBuf, std::io::ErrorKind),

    #[error("Failed to get canonical path of {0}: {1}")]
    GetCanonicalPath(PathBuf, std::io::ErrorKind),

    #[error("Path `{0}` cannot be converted to a string")]
    PathNotString(PathBuf),

    #[cfg_attr(
        feature = "opengraph",
        error("Failed to generate Opengraph image : {0}")
    )]
    #[cfg(feature = "opengraph")]
    GenerateOpengraphImage(opengraph::Error),

    #[error("Failed to write Opengraph image to {0}: {1}")]
    WriteOpengraphImage(PathBuf, std::io::ErrorKind),
}

#[derive(Debug)]
struct CopyAsset {
    source: PathBuf,
    target: PathBuf,
}

#[cfg(feature = "opengraph")]
struct OpengraphPage {
    slug: crate::content::Slug,
    view: AnyView,
}

struct Page {
    view_path: PathBuf,
    view: AnyView,
    #[cfg(feature = "opengraph")]
    opengraph: Option<OpengraphPage>,
}

pub struct Paths {
    pub target: PathBuf,
    #[cfg(feature = "opengraph")]
    pub opengraph: PathBuf,
}

pub struct Blog<'config> {
    paths: Paths,
    config: BuildConfig<'config>,
    pages: Vec<Page>,
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
    pub fn new(paths: Paths, config: config::BuildConfig<'config>) -> Self {
        println!("Building the following configuration: {config:#?}");
        println!("Website will be built in {}", paths.target.display());
        Self {
            paths,
            config,
            pages: vec![],
            assets: vec![],
            atom_feed: None,
        }
    }

    pub fn add_404_page(&mut self, additional_js: fn() -> Option<AnyView>) {
        self.pages.push(Page {
            view_path: format!("{EXTRA_FOLDER}404.html").into(),
            view: pages::not_found_page(self.config, additional_js()),
            #[cfg(feature = "opengraph")]
            opengraph: None,
        });
    }

    pub fn add_index_page(&mut self, content: &[Content], additional_js: fn() -> Option<AnyView>) {
        self.pages.push(Page {
            view_path: format!("{WWW_FOLDER}index.html").into(),
            view: pages::index(content, self.config, additional_js()),
            #[cfg(feature = "opengraph")]
            opengraph: Some(OpengraphPage {
                view: opengraph::template::home(
                    self.config.logo,
                    self.config.website_name,
                    self.config.website_tagline,
                    self.config.absolute_url().as_ref(),
                ),
                slug: "_index".to_string(),
            }),
        });
    }

    pub fn add_content_pages(
        &mut self,
        content: &[Content],
        additional_js: fn() -> Option<AnyView>,
    ) -> Result<(), GenerateHtmlError> {
        struct ProcessedContent {
            slug: crate::content::Slug,
            view: AnyView,

            #[cfg(feature = "opengraph")]
            opengraph: AnyView,
        }

        content
            .iter()
            // Attempt to render content in HTML
            .map(|content| {
                pages::content(content, self.config, additional_js()).map(|view| (content, view))
            })
            // Stop when any failure occurs
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(content, view)| ProcessedContent {
                slug: content.slug(),
                view,

                #[cfg(feature = "opengraph")]
                opengraph: opengraph::template::content(
                    content.meta().title(),
                    self.config.logo,
                    self.config.website_name,
                    self.config.absolute_url().as_ref(),
                ),
            })
            // Add to list of final content
            .for_each(|content| {
                println!("Processed {}", content.slug);
                self.pages.push(Page {
                    view_path: format!("{WWW_FOLDER}{}/index.html", content.slug).into(),
                    view: content.view,

                    #[cfg(feature = "opengraph")]
                    opengraph: Some(OpengraphPage {
                        view: content.opengraph,
                        slug: content.slug,
                    }),
                });
            });

        Ok(())
    }

    fn add_assets(assets: &mut Vec<CopyAsset>, source_base: &Path, target_base: &Path) {
        // Gather list of source assets
        let source_assets = walkdir::WalkDir::new(source_base)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|dir_entry| dir_entry.into_path())
            .filter(|path| path.is_file())
            // Remove markdown file as assets
            .filter(|path| path.extension().map(|ext| ext != "md").unwrap_or(true));

        // For each source asset, get its target path
        let source_and_target_assets = source_assets.filter_map(|source| {
            // Remove source prefix
            source
                .strip_prefix(source_base)
                .map(|path| path.to_path_buf())
                // Add target prefix instead
                .map(|path| target_base.join(path))
                .ok()
                // Keep both source and target paths
                .map(|target| (source, target))
        });

        // Add to the list of assets
        source_and_target_assets.for_each(|(source, target)| {
            println!("Processed asset `{}`", source.display());
            assets.push(CopyAsset { source, target });
        });
    }

    pub fn add_content_assets(&mut self, content_path: impl AsRef<Path>, content: &[Content]) {
        content
            .iter()
            // The names are not the same between content and final folder (since it is public)
            .flat_map(|content| content.assets().map(|assets| (assets, content.slug())))
            .for_each(|(assets, slug)| {
                // Base paths for source and target locations
                let source_base = content_path.as_ref().join(assets);
                let target_base = self.paths.target.join(WWW_FOLDER).join(slug);

                Self::add_assets(&mut self.assets, &source_base, &target_base);
            });
    }

    pub fn add_atom_feed(&mut self, content: &[Content]) {
        self.atom_feed = Some(feed::create_feed(&self.config, content));
    }

    fn write_view_to_file(
        view: AnyView,
        base_path: &Path,
        path: impl AsRef<Path>,
    ) -> Result<PathBuf, BlogWriteFilesError> {
        // Create all parent directories
        let parent = path
            .as_ref()
            .parent()
            .ok_or(BlogWriteFilesError::NoParentFolder(
                path.as_ref().to_path_buf(),
            ))?;
        std::fs::create_dir_all(base_path.join(parent))
            .map_err(|e| BlogWriteFilesError::CreateFolder(e.kind()))?;

        // Write html to file
        let html = RenderHtml::to_html(view);

        #[cfg(not(feature = "optimize"))]
        let html_bytes = html.into_bytes();

        #[cfg(feature = "optimize")]
        let html_bytes = {
            let mut cfg = minify_html::Cfg::new();
            cfg.minify_js = true;

            minify_html::minify(html.as_ref(), &cfg)
        };

        let html_document = base_path.join(path.as_ref());
        std::fs::write(&html_document, html_bytes)
            .map_err(|e| BlogWriteFilesError::WriteFile(html_document.clone(), e.kind()))?;
        println!(
            "wrote `{}` to {}",
            path.as_ref().display(),
            html_document.display()
        );

        Ok(html_document)
    }

    pub fn copy_asset(source: &Path, target: &Path) -> Result<(), BlogWriteFilesError> {
        println!(
            "Copying asset from `{}` to `{}`",
            source.display(),
            target.display()
        );

        // Create parent directory if it does note exist
        let parent = target
            .parent()
            .ok_or(BlogWriteFilesError::NoParentFolder(target.to_path_buf()))?;
        std::fs::create_dir_all(parent).map_err(|e| BlogWriteFilesError::CreateFolder(e.kind()))?;

        // Copy file
        std::fs::copy(source, target)
            .map_err(|e| BlogWriteFilesError::WriteFile(target.to_path_buf(), e.kind()))?;

        Ok(())
    }

    fn write_atom_feed(atom_feed: Feed, target: &Path) -> Result<PathBuf, BlogWriteFilesError> {
        let path = target.join(format!("{WWW_FOLDER}atom.xml"));
        std::fs::write(&path, atom_feed.to_string())
            .map_err(|e| BlogWriteFilesError::WriteFile(path.clone(), e.kind()))?;

        Ok(path)
    }

    /// Consume struct, write to files
    pub fn build(mut self) -> Result<PathBuf, BlogWriteFilesError> {
        for Page {
            view_path,
            view,
            #[cfg(feature = "opengraph")]
            opengraph,
        } in self.pages
        {
            // Render opengraph images
            #[cfg(feature = "opengraph")]
            if let Some(opengraph) = opengraph {
                // Write html view to file
                let opengraph_html_path = Self::write_view_to_file(
                    opengraph.view,
                    self.paths.opengraph.as_path(),
                    format!("{}.html", opengraph.slug),
                )?;
                println!("opengraph: wrote template for {}", opengraph.slug);

                let logo: PathBuf = format!("{}{}", self.config.assets, self.config.logo).into();
                let logo_for_opengraph = self.paths.opengraph.join(self.config.logo);
                std::fs::copy(&logo, &logo_for_opengraph)
                    .map_err(|e| BlogWriteFilesError::CopyFile(logo.clone(), e.kind()))?;
                println!(
                    "Copied `{}` to `{}`",
                    logo.display(),
                    logo_for_opengraph.display()
                );

                // Copy CSS stylesheet used in opengraph templates
                let opengraph_style = self.paths.opengraph.join("opengraph_style.css");
                std::fs::copy(self.config.styles.opengraph, &opengraph_style).map_err(|e| {
                    BlogWriteFilesError::CopyFile(self.config.styles.opengraph.into(), e.kind())
                })?;
                println!(
                    "Copied `{}` to `{}`",
                    self.config.styles.opengraph,
                    opengraph_style.display()
                );

                let opengraph_html_url = opengraph_html_path
                    .canonicalize()
                    .map_err(|e| {
                        BlogWriteFilesError::GetCanonicalPath(opengraph_html_path.clone(), e.kind())
                    })?
                    .to_str()
                    .map(|path| format!("file:///{path}"))
                    .ok_or(BlogWriteFilesError::PathNotString(
                        opengraph_html_path.clone(),
                    ))?;

                // Open html files and take a screenshot
                let screenshot =
                    opengraph::export_view_to_png(&opengraph_html_url, self.config.webdriver)
                        .map_err(BlogWriteFilesError::GenerateOpengraphImage)?;

                // Write screenshot to a file
                let opengraph_png_path = self
                    .paths
                    .opengraph
                    .join(format!("{}.png", opengraph.slug.as_str()));
                std::fs::write(&opengraph_png_path, screenshot).map_err(|e| {
                    BlogWriteFilesError::WriteOpengraphImage(opengraph_png_path.clone(), e.kind())
                })?;

                let target = {
                    let filename = match opengraph.slug.as_ref() {
                        "_index" => "opengraph.png".into(),
                        _ => format!("{}/opengraph.png", opengraph.slug),
                    };
                    self.paths.target.join(WWW_FOLDER).join(filename)
                };

                // Add opengraph image to assets to copy
                self.assets.push(CopyAsset {
                    source: opengraph_png_path,
                    target,
                });
            }

            // Render views to HTML files
            Self::write_view_to_file(view, self.paths.target.as_path(), &view_path)?;
        }

        // Add internal assets
        Self::add_assets(
            &mut self.assets,
            PathBuf::from(self.config.assets).as_path(),
            self.paths.target.join(WWW_FOLDER).as_path(),
        );

        // Copy content assets + internal assets
        for copy_asset in self.assets {
            Self::copy_asset(&copy_asset.source, &copy_asset.target)?;
        }

        // Atom feed
        if let Some(atom_feed) = self.atom_feed {
            Self::write_atom_feed(atom_feed, &self.paths.target)?;
        }

        Ok(self.paths.target)
    }
}
