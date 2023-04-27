mod lua;
mod script;

use std::fmt::Display;

use lua::run_lua_minion;
use std::error::Error;
use serde::{Deserialize, Serialize};

use crate::ProgramInfo;
pub use script::*;

#[derive(Debug, Clone)]
pub struct MinionError(pub String);

impl Display for MinionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MinionError: {}", self.0)
    }
}

impl Error for MinionError {}

#[derive(Serialize, Deserialize, Clone)]
pub struct MinionResponse {
    pub findings: Vec<String>,
    pub changes: Vec<String>
}

fn to_points(points: &[String]) -> String {
    points.iter()                
        .map(|el| format!("- {el}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn create_letter(findings: &[String], changes: &[String]) -> String {
    format!(
        "Dear Boss,
        
        I have completed the tasks you assigned to me. These are my findings:
        {}
        
        These are the changes I had to carry out:
        {}
        
        Sincerely, Your Employee.",
        to_points(findings),
        to_points(changes)
    )
}

pub fn run_minion(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<(String, MinionResponse), Box<dyn Error>> {
    run_lua_minion(program, task, new_prompt)
}
