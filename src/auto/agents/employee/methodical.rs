use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, run::Action}};

use super::log_yaml;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalThoughts {
    pub thoughts: String,
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalAction {
    pub tool: String,
    pub purpose: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalStep {
    pub idea: String,
    pub action: MethodicalAction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalPlan {
    pub steps: Vec<MethodicalStep>
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    first_inst: bool
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    if first_inst {
        agent.llm.prompt.push(Message::User(format!(r#"
Tools:
google_search {{ "query": "..." }}
browse_urls {{ "urls": [ "..." ] }}
file_append {{ "path": "...", "content": "..." }}

You have been given these tools.

Task:
Research at least three articles on M&M health detriments.

Create a list of steps.
Each step will use one tool.
Then, describe each time you will use the tool (i.e. browsing multiple articles)
Do not specify arguments.

Respond in this JSON format:
```json
{{
    "steps": [
        {{
            "idea": "idea",
            "action": {{
                "tool": "tool",
                "purpose": "purpose"
            }}
        }}
    ]
}}
```
"#)));
    }

    let plan = try_parse_json::<MethodicalPlan>(&agent.llm, 2, Some(300))?;
    agent.llm.prompt.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    log_yaml(&plan)?;



    for step in plan.steps {
        let step_text = serde_yaml::to_string(&step)?;

        agent.llm.prompt.push(Message::User(format!(r#"
You have created a plan.
Now you will carry out the first step: 
{step_text}

Respond in this JSON format:
```json
{{
    "thoughts: "thoughts",
    "action": {{
        "tool": "tool",
        "args": {{ ... }}
    }}
}}
```
"#)))
    }

    Ok("No".to_string())
}