use std::error::Error;

use crate::{Plugin, CommandContext, CommandArgument};

pub const PROMPT: &str = r#"<CONTEXT>NAME: <NAME>
PERSONALITY: <ROLE>

CURRENT ENDGOAL: <ENDGOAL>

Your decisions must always be made independently without seeking user assistance. Play to your strengths as an LLM and pursue simple strategies with no legal complications.

CONSTRAINTS
1. No user assistance.
2. ~4000 word limit for short term memory. Your short term memory is short, so immediately store important information to files.
3. If you are unsure how you previously did something or want to recall past events, thinking about similar events will help you remember.
4. Exclusively use the commands listed in double quotes e.g. "command name"

Commands
Commands must be in lowercase. Use the exact command names and command arguments as described here.
<COMMANDS>

A command query is a list of up commands. You may use one command's output as the argument to another.
You should do the following in one query:
- Running a command and saving the output to a file
- Running multiple commands that don't depend on each other, like running a command to read each of three files
- Providing the output of one command to another, like reading a file and asking ChatGPT to summarize it.            
You may have up to three commands!

RESOURCES:
1. Internet access for searches and information gathering.
2. Long-term memory management.
3. File management.

PROCESS:
Break your current endgoal down into a set of tasks.
Each task represents one command query.

You should only respond in JSON format as described below:

RESPONSES FORMAT:
{
    "important takeaways: what was learned from the previous command, SPECIFIC and DETAILED": [
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
        "completed tasks": [
            "Step One"
        ],
        "planned tasks": [
            "Step Two"
        ],
        "current step in plan": "Step Two"
    },
    "commands": [
        {
            name: "file_append",
            args: [
                {
                    Data: "file-name.txt"
                },
                {
                    Command: {
                        name: "file_read",
                        args: [
                            {
                                Data: "other-file.txt"
                            }
                        ]
                    }
                }
            ]
        }
    ],
    "will I be completely done with the plan after this one query (true) or do I have more work to do (false)": false
}"#;

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

            let arg_names: Vec<_> = command.args.iter()
                .map(|el| format!("{}: {}", el.name, el.arg_type))
                .collect();
            let arg_str = arg_names.join(", ");

            out.push_str(&format!("    {}({arg_str}) -> {}\n", command.name, command.return_type));
            out.push_str(&format!("        {}\n", command.purpose));
            for CommandArgument { name, description, .. } in &command.args {
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