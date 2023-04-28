mod lua;
mod continuous;
mod script;
mod prompts;

use std::fmt::Display;

use lua::run_lua_minion;
use continuous::run_continuous_minion;
use std::error::Error;
use serde::{Deserialize, Serialize};

use crate::ProgramInfo;
pub use script::*;
pub use prompts::*;

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

pub fn run_minion(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<(String, MinionResponse), Box<dyn Error>> {
    run_continuous_minion(program, task, new_prompt)
}
