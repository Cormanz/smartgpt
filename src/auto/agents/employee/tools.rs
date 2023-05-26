use crate::{Command, CommandArgument};

pub fn create_tool_list(commands: &[&Command]) -> String {
    let mut prompt = format!("Tools:");

    for command in commands {
        let Command { name, purpose, args, .. } = command;

        let mut schema = format!("{{ ");
        for arg in args {
            let CommandArgument { name, example } = arg;
            schema.push_str(&format!(r#""{name}": {example}, "#))
        }
        schema = schema.trim_end_matches(", ").to_string();

        schema.push_str(&format!(" }}"));

        prompt.push('\n');
        prompt.push_str(&format!("{name} {schema} - {purpose}"));
    }

    return prompt;
}