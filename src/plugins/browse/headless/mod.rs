use std::path::Path;
use std::{path::PathBuf, process::exit};

use anyhow::Result;
// use article_scraper::{
//     ArticleScraper, FtrConfigEntry, FullTextParser,
//     Readability::{self},
// };
use headless_chrome::{
    protocol::cdp::Page::CaptureScreenshotFormatOption, types::PrintToPdfOptions, Browser,
    LaunchOptions,
};
use poppler::PopplerDocument;
use poppler::PopplerPage;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::fs;
use tokio::sync::mpsc::{self, Sender};
use url::Url;

pub async fn get_content_headless(url: &str) -> anyhow::Result<String> {
    let options = LaunchOptions {
        headless: true,
        window_size: Some((1200, 1920)),
        ..Default::default()
    };

    let browser = Browser::new(options)?;

    // let url = "https://github.com/Cormanz/smartgpt";
    // let url = "https://github.com/Cormanz/smartgpt/tree/main/src";

    let tab = browser.new_tab()?;
    tab.set_default_timeout(std::time::Duration::from_secs(10));
    tab.navigate_to(url)?;

    let html = tab.get_content()?;

    let parsed_url = Url::parse(&url)?;
    let base_url = format!(
        "{}://{}",
        parsed_url.scheme(),
        parsed_url.host_str().unwrap()
    );
    // let base_url = Url::parse("https://github.com/").unwrap();

    let pdf_options: Option<PrintToPdfOptions> = Some(PrintToPdfOptions {
        landscape: Some(false),
        display_header_footer: Some(false),
        print_background: Some(false),
        scale: Some(0.5),
        paper_width: Some(11.0),
        paper_height: Some(17.0),
        margin_top: Some(0.1),
        margin_bottom: Some(0.1),
        margin_left: Some(0.1),
        margin_right: Some(0.1),
        page_ranges: Some("1-2".to_string()),
        ignore_invalid_page_ranges: Some(true),
        prefer_css_page_size: Some(false),
        transfer_mode: None,
        ..Default::default()
    });

    let pdf_data = tab.print_to_pdf(pdf_options)?;

    // fs::write("github.pdf", pdf_data.clone())?;

    let mut pdf_as_vec = pdf_data.to_vec();

    let doc = PopplerDocument::new_from_data(&mut pdf_as_vec, "")?;

    let page = doc.get_page(0).unwrap();
    match page.get_text() {
        Some(content) => Ok(content.to_string()),
        None => Err(anyhow::anyhow!("failed to parse pdf to text")),
    }

    // println!("page {:?}", content);

    // fs::write("raw.github.html", html.clone())?;

    // println!("{:?}", extracted_content.);
    // let bytes = std::fs::read("new-ithub.pdf")?;
    // let out = pdf_extract::extract_text_from_mem(&bytes)?;
}
