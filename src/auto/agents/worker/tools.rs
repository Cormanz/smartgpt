use std::mem::discriminant;

use crate::{Tool, ToolArgument, ToolType};

pub fn create_filtered_tool_list(header: &str, tools: &[&Tool], tool_type: ToolType) -> String {
    let mut prompt = header.to_string();

    for tool in tools {
        // Compares if the tool types are the same, not the values of the tool types
        if discriminant(&tool.tool_type) == discriminant(&tool_type) {
            continue;
        }

        let Tool { name, purpose, args, .. } = tool;

        let mut schema = format!("{{ ");
        for arg in args {
            let ToolArgument { name, example } = arg;
            schema.push_str(&format!(r#""{name}": {example}, "#))
        }
        schema = schema.trim_end_matches(", ").to_string();

        schema.push_str(&format!(" }}"));

        prompt.push('\n');
        prompt.push_str(&format!("{name} {schema} - {purpose}"));
    }

    return prompt;
}

pub fn create_tool_list(tools: &[&Tool]) -> String {
    vec![
        create_filtered_tool_list("Resources", tools, ToolType::Resource),
        create_filtered_tool_list("Actions", tools, ToolType::Action { needs_permission: false })
    ].join("\n\n")
}