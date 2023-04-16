mod manager;
mod boss;
mod employee;

pub use manager::*;
pub use boss::*;
pub use employee::*;
use serde::{Deserialize, Serialize};

pub const LINE_WRAP: usize = 12;

pub fn process_response(text: &str, line_wrap: usize) -> String {
    let lines: Vec<String> = text.split("\n")
        .flat_map(|line| line.split(" ")
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
            .chunks(line_wrap)
            .map(|el| el.join(" "))
            .collect::<Vec<_>>()
        )
        .map(|el| format!("    {el}"))
        .collect();
    lines.join("\n")
}

#[derive(Serialize, Deserialize)]
pub struct Choice {
    pub choice: String
}