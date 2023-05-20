// use std::path::Path;
// use std::{path::PathBuf, process::exit};

use anyhow::Result;
// use article_scraper::{
//     ArticleScraper, FtrConfigEntry, FullTextParser,
//     Readability::{self},
// };
use headless_chrome::{
    protocol::cdp::Page::CaptureScreenshotFormatOption, types::PrintToPdfOptions, Browser,
    LaunchOptions,
};
// use html2text::from_read;
use poppler::PopplerDocument;
use poppler::PopplerPage;
// use std::fs::{write, File};
// use std::io::{self, BufReader};
// use tokio::sync::mpsc::{self, Sender};
use url::Url;

// this webpage parsing method relies on poppler-rs, a library for rendering PDF files
// Warning: poppler depends on libpoppler, which has only been tested on Linux
// ensure that libpoppler-glib is installed to use it.
pub async fn get_content_headless(url: &str) -> anyhow::Result<String> {
    // set the headless Chrome to open a webpage in portrait mode of certain width and height
    // could possibly lower the likelyhood of loading unwanted/lower priority elements
    let options = LaunchOptions {
        headless: true,
        window_size: Some((1200, 1920)),
        ..Default::default()
    };

    let browser = Browser::new(options)?;

    // let mut text_from_readability_parsing = String::new();
    let mut text_from_pdf_conversion = String::new();

    let tab = browser.new_tab()?;

    // giving the browser 5 seconds to load a page
    tab.set_default_timeout(std::time::Duration::from_secs(5));
    tab.navigate_to(url)?;

    let html = tab.get_content()?;

    let parsed_url = Url::parse(&url)?;
    let host_str = parsed_url.host_str().unwrap_or("");
    let base_url = format!("{}://{}", parsed_url.scheme(), host_str);
    let base_url = Url::parse(&base_url)?;
    // let base_url = Url::parse("https://github.com/").unwrap();
    // let raw_file_name = format!("output/raw.{host_str}.html");
    // let clean_file_name = format!("output/clean.{host_str}.html");
    // write(&raw_file_name, html.clone())?;

    // match Readability::extract(&html, Some(base_url)).await {
    //     Ok(res) => {
    //         // println!("{:?}", res.to_string());
    //         text_from_readability_parsing = from_read(res.to_string().as_bytes(), 80);
    //         println!("{:?}", text_from_readability_parsing.to_string());
    //     }
    //     Err(_err) => {}
    // };

    // match std::fs::write(PathBuf::from(&clean_file_name), result.clone()) {
    //     Ok(()) => {println!("{:?}", result.clone());
    // return Ok(result);}
    //     Err(err) => {
    //        exit(0);
    //     }
    // }

    // print the webpage to a long paper virtually, shrink the print scale, hoping to get
    // important information in this single page
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

    // // fs::write("github.pdf", pdf_data.clone())?;

    let mut pdf_as_vec = pdf_data.to_vec();

    // parse captured webpage in pdf format, with "", which means no password
    let doc = PopplerDocument::new_from_data(&mut pdf_as_vec, "")?;
    // println!("----------------------");

    // work with the one and only page, obtain the text data
    if let Some(page) = doc.get_page(0) {
        if let Some(content) = page.get_text() {
            text_from_pdf_conversion = content.to_string();
            // println!("{:?}", text_from_pdf_conversion.to_string());
        }
    };
    Ok(text_from_pdf_conversion)

    // if text_from_readability_parsing.split_whitespace().count() > 300 {
    //     // Ok(text_from_readability_parsing)
    //     Ok(text_from_pdf_conversion)
    // } else {
    //     // Ok(text_from_pdf_conversion)
    //     Ok(text_from_readability_parsing)
    // }
    // None => Err(anyhow::anyhow!("failed to parse pdf to text")),
}
