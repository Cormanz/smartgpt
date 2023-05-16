use std::error::Error;

use colored::Colorize;
use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{run::Action, try_parse_json, agents::findings::{get_observations, ask_for_findings}}, Weights};

use super::use_tool;

pub fn log_yaml<T: Serialize>(data: &T) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_yaml::to_string(&data)?);

    Ok(())
}

pub fn create_main_task_msg(task: &str) -> Message {
    Message::User(format!(r#"
Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "..." }}
browse_url {{ "url": "..." }}
    Only use browse_url on websites that you have found from other searches.
file_append {{ "path": "...", "content": "..." }}

Task:
{task}

Your goal is to complete your task by running actions.
You will decide whether or not you have completed your task through a detailed analysis of at least one sentence.

If you have not, begin having "thoughts".
Your "thoughts", "reasoning", and "criticism" are ideas of HOW you will complete your task.

You must use at least one action before completing the task.

Only focus on your task. Do not try to do more then what you are asked.

```json
{{
    "what have I done so far": "progress",
    "explanation of why or why not my task is complete": "explanation",
    "if my task is not complete, how do I finish it soon": null / "...",
    "thoughts about completing my task": "thought",
    "reasoning": "reasoning",
    "criticism": "constructive self-criticism",
    "action": {{ 
        "tool": "...",
        "args": {{ ... }}
    }}
}}
```

"action" may only be `null` if the task is complete.

Respond in the above JSON format exactly.
Ensure every field is filled in.
"#
    ))
}

#[derive(Serialize, Deserialize)]
pub struct Thoughts {
    #[serde(rename = "what have I done so far")]
    pub progress: String,
    #[serde(rename = "explanation of why or why not my task is complete")]
    pub explanation: String,
    #[serde(rename = "if my task is not complete, how do I finish it soon")]
    pub soon: Option<String>,
    #[serde(rename = "thoughts about completing my task")]
    pub thoughts: Option<String>,
    pub reasoning: Option<String>,
    pub criticism: Option<String>,
    pub action: Option<Action>
}

pub fn explain_results(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
   agent.llm.message_history.push(Message::System(format!(
"Now that you have finished your task, write a detailed, readable and simple Markdown response.
Your response should be easily understandable for a human, and convey all information in an accessible format.
Ensure that sources are linked in the Markdown representation.
Respond in exact plaintext; no JSON.
Keep your response at four paragraphs or less."
    )));

    agent.llm.model.get_response_sync(&agent.llm.get_messages(), Some(600), None)
}

pub fn get_thoughts(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
) -> Result<Thoughts, Box<dyn Error>> {
    Ok(
        try_parse_json(&get_agent(context).llm, 2, Some(600))
            .map(|res: crate::auto::ParsedResponse<Thoughts>| {
                get_agent(context).llm.message_history.push(Message::Assistant(res.raw));
                res.data
            })?
    )
}

pub enum ActionResults {
    TaskComplete(String),
    Results(String)
}

pub fn run_react_action(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<ActionResults, Box<dyn Error>> {
    let thoughts: Thoughts = get_thoughts(context, get_agent)?;
    log_yaml(&thoughts)?;

    match thoughts.action {
        Some(action) => {
            Ok(ActionResults::Results(
                use_tool(context, &|context| &mut context.agents.fast, action)?
            ))
        }
        None => {
            Ok(ActionResults::TaskComplete(
                explain_results(context, &get_agent)?
            ))
        }
    }
}

pub fn run_react_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    first_inst: bool
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    if first_inst {
        agent.llm.prompt.push(Message::System(format!("
You are an Agent.
You will complete your task, one action at a time.
"
        )));

        agent.llm.prompt.push(Message::User(format!("
Long Term Observations:
None found.

Take advantage of your long-term observations as much as possible."
        )));

        agent.llm.prompt.push(create_main_task_msg(task));
    } else {
        agent.llm.prompt[2] = create_main_task_msg(task);
        agent.llm.message_history.push(Message::User(format!(
r#"
You have been given a new task:
{task}

Complete this task just as before.
Respond in this precise JSON format:

```json
{{
    "what have I done so far": "progress",
    "explanation of why or why not my task is complete": "explanation",
    "if my task is not complete, how do I finish it soon": null / "...",
    "thoughts about completing my task": "thought",
    "reasoning": "reasoning",
    "criticism": "constructive self-criticism",
    "action": {{ 
        "tool": "...",
        "args": {{ ... }}
    }}
}}
```
"#
        )));
    }

    loop {
        let agent = get_agent(context);

        let remaining_tokens = agent.llm.get_tokens_remaining(
            &agent.llm.get_messages()
        )?;

        if remaining_tokens < 1000 {
            ask_for_findings(agent)?;

            let observations = get_observations(agent, task, 20, Weights {
                recall: 1.,
                recency: 1.,
                relevance: 1.
            })?
                .unwrap_or("None found.".to_string());

            agent.llm.prompt[1] = Message::User(format!("
Long Term Observations:
{observations}

Take advantage of your long-term observations as much as possible."));

            agent.llm.crop_to_tokens_remaining(2200)?;
        }

        drop(agent);
        let results = run_react_action(context, get_agent, task)?;
        let agent = get_agent(context);

        match results {
            ActionResults::Results(results) => {
                println!("{results}");

                agent.llm.message_history.push(Message::User(format!(
r#"Your tool use gave the following result:

{results}

Please decide on your next action to complete your initial task of '{task}'.
Only focus on your task, do not get tunnel vision."#
                )));
            },
            ActionResults::TaskComplete(completion_message) => {
                return Ok(completion_message);
            }
        }

        println!();
    }
}