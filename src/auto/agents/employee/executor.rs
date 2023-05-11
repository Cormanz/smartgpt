use std::error::Error;

use tokio::runtime::Runtime;

use crate::{ProgramInfo, auto::run::run_command};

use super::brainstormer::Action;

pub fn execute(program: &mut ProgramInfo, action: Action) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { context, plugins, .. } = program;
    let mut context = context.lock().unwrap();

    let command = plugins.iter()
        .flat_map(|el| &el.commands)
        .find(|el| el.name == action.tool);

    let mut out = String::new();
    match command {
        Some(command) => {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                run_command(
                    &mut out, 
                    action.tool.clone(), 
                    command.box_clone(), 
                    &mut context, 
                    action.args
                ).await
            })?;

        },
        None => {
            let error_str = format!("No such tool named '{}.'", action.tool);
            out.push_str(&error_str)
        }
    }

    Ok(out)
}