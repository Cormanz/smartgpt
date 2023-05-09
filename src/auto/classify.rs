use std::error::Error;

use crate::ProgramInfo;

pub fn classify(program: &mut ProgramInfo) -> Result<String, Box<dyn Error>> {
    Ok("a".to_string())
}