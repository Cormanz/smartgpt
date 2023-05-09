use std::error::Error;

use crate::{Plugin, CommandContext};

pub fn generate_commands(plugins: &[Plugin], disabled_commands: &[String]) -> String {
    let mut out = String::new();
    for plugin in plugins {
        for command in &plugin.commands {
            if disabled_commands.contains(&command.name) {
                continue;
            }

            let arg_names: Vec<_> = command.args.iter()
                .map(|el| format!("{}: {}", el.name, el.arg_type))
                .collect();
            let arg_str = arg_names.join(", ");

            out.push_str(&format!("    {}({arg_str}) -> {}\n", command.name, command.return_type));
            out.push_str(&format!("        {}\n", command.purpose));
            /*for CommandArgument { name, description, .. } in &command.args {
                out.push_str(&format!("            - {}: {}\n", name, description)); 
            }*/
        }
    }
    out.trim_end().to_string()
}

pub fn generate_commands_short(plugins: &[Plugin], disabled_commands: &[String]) -> String {
    let mut out = String::new();
    for plugin in plugins {
        for command in &plugin.commands {
            if disabled_commands.contains(&command.name) {
                continue;
            }

            let arg_names: Vec<_> = command.args.iter()
                .map(|el| format!("{}: {}", el.name, el.arg_type))
                .collect();
            let arg_str = arg_names.join(", ");

            out.push_str(&format!("    {}({arg_str}), ", command.name));
            /*for CommandArgument { name, description, .. } in &command.args {
                out.push_str(&format!("            - {}: {}\n", name, description)); 
            }*/
        }
    }
    out.trim_end().to_string()
}

pub async fn generate_context(context: &mut CommandContext, plugins: &[Plugin], previous_prompt: Option<&str>) -> Result<String, Box<dyn Error>> {
    let mut out: Vec<String> = vec![];
    for plugin in plugins {
        let context = plugin.cycle.create_context(context, previous_prompt).await?;
        if let Some(context) = context {
            out.push(context);
        }
    }

    Ok(if out.len() > 0 {
        out.join("\n\n") + "\n\n"
    } else {
        "".to_string()
    })
}