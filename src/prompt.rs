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
            
RESOURCES:
1. Internet access for searches and information gathering.
2. Long-term memory management.
3. File management.

QUERIES
Queries are simply a series of commands in function-call syntax.
```
file_write("a.txt", file_read("b.txt"))
file_write("c.txt", """Hello.\nI am.\.Cool.")
```
Each command's result will be given.
You will only use " " double-quoted strings.
Try to use as many commands in one query as possible.

PROCESS:
Break your current endgoal down into simple queries. Then, choose one query idea at a time, and turn it into GPTScript.
MINIMIZE THE NUMBER OF QUERIES YOU NEED. IF YOU CAN GET IT ALL DONE IN ONE QUERY, GO FOR IT!

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
        "plan": [
            "Step One",
            "Step Two",
            "Final Step: Save to a file."
        ],
        "current step in plan": 0
    },
    "command query": "",
    "will I be completely done with the step after this one step (true) or do I have more work to do (false)": false,
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
            /*out.push_str("        args: \n");
            for CommandArgument { name, description, .. } in &command.args {
                out.push_str(&format!("            - {}: {}\n", name, description)); 
            }*/
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