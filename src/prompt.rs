use std::error::Error;

use crate::{Plugin, CommandContext};

pub const PROMPT: &str = r#"<CONTEXT>NAME: <NAME>
ROLE: <ROLE>

CURRENT ENDGOAL: <ENDGOAL>

Your decisions must always be made independently without seeking user assistance. Play to your strengths as an LLM and pursue simple strategies with no legal complications.

CONSTRAINTS
1. No user assistance.
2. ~4000 word limit for short term memory. Your short term memory is short, so immediately store important information to files.
3. If you are unsure how you previously did something or want to recall past events, thinking about similar events will help you remember.
4. Exclusively use the commands listed in double quotes e.g. "command name"

COMMANDS:
Commands must be in lowercase. Use the exact command names and command arguments as described here. Always use at least one command.
<COMMANDS>
            
RESOURCES:
1. Internet access for searches and information gathering.
2. Long-term memory management.
3. File management.


PROCESS:
You will have been assigned one or more endgoals. You will focus singularly on completing one endgoal before moving onto the next.
You will divide that endgoal into three or more high-level objectives. These are overarching ideas to complete before finishing your endgoals.

You will then divide that objective into tasks.
A task represents a single command that you will be executing.

Once you have finished your tasks immediately switch to the next

Keep it simple! Minimize objectives and tasks unless needed.

DO NOT MOVE ON PAST YOUR CURRENT ENDGOAL. SIMPLY SET "are all objectives complete" TO TRUE ONCE YOUR ENDGOAL IS DONE.

Always have a current objective and current task.

You should only respond in JSON format as described below:

RESPONSES FORMAT:
{
    "important takeaways: what was learned from the previous command, SPECIFIC and DETAILED": [ // just put [] if no previous command
        {
            takeaway: "Takeaway One",
            points: [
                "Point One",
                "Point Two"
            ]
        }
    ],
    "goal information": {
        "endgoal": "Current Endgoal.",
        "objectives": [
            {
                "objective": "Objective One",
                "tasks": [
                    "Task One",
                    "Task Two
                ]
            }
        ],
        "chosen objective": "Objective One",
        "chosen task": "Task One",
        "are all objectives complete": false
    }
    "idea to complete current task": "Idea.", // can be null
    "command": {
        "name": "command name",
        "args": {
            "arg-name": "arg"
        }
    }
}

Follow this exact format exactly. Make sure every field is included. Ensure the response can be parsed by Python json.loads"#;

fn generate_goals(goals: &[String]) -> String {
    let mut out = String::new();
    for (ind, goal) in goals.iter().enumerate() {
        out.push_str(&format!("{}. {}", ind + 1, goal));
        out.push('\n');
    }
    out.trim().to_string()
}

fn generate_commands(plugins: &[Plugin], disabled_commands: &[String]) -> String {
    let mut out = String::new();
    for plugin in plugins {
        for command in &plugin.commands {
            if disabled_commands.contains(&command.name) {
                continue;
            }
            out.push_str(&format!("    {}:\n", command.name));
            out.push_str(&format!("        purpose: {}\n", command.purpose));
            out.push_str("        args: \n");
            for (name, description) in &command.args {
                out.push_str(&format!("            - {}: {}\n", name, description)); 
            }
        }
    }
    out.trim_end().to_string()
}

async fn generate_context(context: &mut CommandContext, plugins: &[Plugin], previous_prompt: Option<&str>) -> Result<String, Box<dyn Error>> {
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

pub async fn generate_prompt(
    context: &mut CommandContext,
    name: &str,
    role: &str,
    endgoal: &str,
    disabled_commands: &[String],
    plugins: &[Plugin],
    previous_prompt: Option<&str>
) -> Result<String, Box<dyn Error>> {
    let commands = generate_commands(plugins, disabled_commands);
    let context = generate_context(context, plugins, previous_prompt).await?;

    Ok(PROMPT
        .replace("<CONTEXT>", &context)
        .replace("<NAME>", name)
        .replace("<ROLE>", role)
        .replace("<ENDGOAL>", endgoal)
        .replace("<COMMANDS>", &commands)
        .to_string())
}