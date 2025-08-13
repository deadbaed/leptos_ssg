pub mod template {
    use leptos::prelude::*;
    use tailwind_fuse::tw_join;

    pub fn home(logo: &str, website_name: &str, website_tagline: &str, url: &str) -> AnyView {
        view! {
        <link rel="stylesheet" href="./opengraph_style.css" />
        <div id="opengraph" class=tw_join!("h-[630px]", "w-[1200px]", "p-24", "border-4", "border-solid", "bg-gray-300")>
            <div class=tw_join!("flex", "h-full", "w-full", "flex-col", "items-stretch", "justify-between")>
                <div class=tw_join!("flex", "flex-row", "space-x-16")>
                    <img src=format!("./{}", logo) class=tw_join!("max-w-80", "border-1", "bg-gray-400") />
                    <div class=tw_join!("flex", "w-full", "flex-col", "space-y-8")>
                        <div class=tw_join!("text-7xl", "font-bold")>{website_name}</div>
                        <div class=tw_join!("text-4xl", "font-semibold")>{website_tagline}</div>
                    </div>
                </div>
                <div class=tw_join!("text-4xl", "font-medium")>{url}</div>
            </div>
        </div>
        }.into_any()
    }

    pub fn content(title: &str, logo: &str, website_name: &str, url: &str) -> AnyView {
        view! {
        <link rel="stylesheet" href="./opengraph_style.css" />
        <div id="opengraph" class=tw_join!("h-[630px]", "w-[1200px]", "p-24", "border-4", "border-solid", "bg-gray-300")>
            <div class=tw_join!("flex", "h-full", "w-full", "flex-col", "items-stretch", "justify-between")>
                <div class=tw_join!("text-7xl", "font-bold")>{title}</div>
                <div class=tw_join!("flex", "flex-row", "space-x-8")>
                    <img src=format!("./{}", logo) class=tw_join!("w-42", "border-1", "bg-gray-400") />
                    <div class=tw_join!("flex", "w-full", "flex-col", "justify-evenly")>
                        <div class=tw_join!("text-5xl", "font-semibold")>{website_name}</div>
                        <div class=tw_join!("text-4xl", "font-medium")>{url}</div>
                    </div>
                </div>
            </div>
        </div>
        }.into_any()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create runtime: {0}")]
    Runtime(std::io::ErrorKind),
    #[error("Failed to connect to webdriver `{0}`: {1}")]
    StartWebdriver(String, fantoccini::error::NewSessionError),
    #[error("Failed to go to url `{0}`: {1}")]
    Goto(String, fantoccini::error::CmdError),
    #[error("Faailed to wait for element `{0:#?}`: {1}")]
    WaitForElement(fantoccini::Locator<'static>, fantoccini::error::CmdError),
    #[error("Failed to take screenshot of element `{0:#?}`: {1}")]
    Screenshot(fantoccini::Locator<'static>, fantoccini::error::CmdError),
    #[error("Failed to finish webdriver session: {0}")]
    EndWebdriver(fantoccini::error::CmdError),

    #[cfg_attr(feature = "optimize", error("Failed to optimize screenshot size: {0}"))]
    #[cfg(feature = "optimize")]
    Optimize(oxipng::PngError),
}

pub fn export_view_to_png(html_view: &str, webdriver: &str) -> Result<Vec<u8>, Error> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| Error::Runtime(e.kind()))?;

    let screenshot = rt.block_on(async {
        let client = fantoccini::ClientBuilder::native()
            .connect(webdriver)
            .await
            .map_err(|e| Error::StartWebdriver(webdriver.to_string(), e))?;

        client
            .goto(html_view)
            .await
            .map_err(|e| Error::Goto(html_view.to_string(), e))?;

        let element = fantoccini::Locator::Id("opengraph");
        let div = client
            .wait()
            .for_element(element)
            .await
            .map_err(|e| Error::WaitForElement(element, e))?;

        let screenshot = div
            .screenshot()
            .await
            .map_err(|e| Error::Screenshot(element, e))?;

        // Then close the browser window.
        client.close().await.map_err(Error::EndWebdriver)?;

        Ok(screenshot)
    })?;

    // Optimize PNG
    #[cfg(feature = "optimize")]
    let screenshot = oxipng::optimize_from_memory(&screenshot, &oxipng::Options::max_compression())
        .map_err(Error::Optimize)?;

    Ok(screenshot)
}
