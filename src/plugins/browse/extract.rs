use std::{error::Error, collections::HashMap};

use reqwest::Client;
use select::{document::Document, predicate::{Name, Or}};
use serde::{Serialize, Deserialize};

pub fn extract_text_from_html(html: &str) -> String {
    let mut text = String::new();

    let document = Document::from(html);

    for p in document.find(Name("p")) {
        text.push_str(&p.text());
    }
    // prints "This is some text."

    text
        .trim()
        .replace(|c: char| !c.is_ascii(), "")
        .to_string()
}