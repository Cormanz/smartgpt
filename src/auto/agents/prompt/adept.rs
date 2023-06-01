use std::marker::PhantomData;

use serde::{Serialize, Deserialize};

use super::Prompt;

#[derive(Serialize, Deserialize)]
pub struct PersonalityInfo {
    pub personality: String
}

pub const PERSONALITY: Prompt<PersonalityInfo> = Prompt("Personality: [personality]", PhantomData);

#[derive(Serialize, Deserialize)]
pub struct ConcisePlanInfo {
    pub task: String
}

pub const CONCISE_PLAN: Prompt<ConcisePlanInfo> = Prompt(r#"
This is your task:
[task]

Make a concise, one-sentence plan on you can complete this task.
Remember that you have access to external tools, so you can do any task.

Respond in this JSON format:
```json
{{
	"concise plan on how you will complete the task": "plan"
}}
```
"#, PhantomData);

#[derive(Serialize, Deserialize)]
pub struct ThoughtInfo {
    pub plan: String,
    pub assets: String
}

pub const THOUGHTS: Prompt<ThoughtInfo> = Prompt(r#"
Your goal is to complete the task by spawning agents to complete smaller subtasks.
Focus on using thoughts, reasoning, and self-criticism to complete your goals.

You make a decision. Here are the types of decisions alongside their `args` schema:

spawn_agent {{ "subtask": "subtask in natural language with all context and details", "assets": [ "asset_name" ], "desired_response": "all specific information desired" }} - Delegate a task to the Agent. Keep it simple.
brainstorm {{ "lines": [ "line 1", "line 2" ] }} - Brainstorm an idea, or generate a response based on the information given yourself.
final_response {{ "response": "response" }} - Give a response to the user.

Assets:
[assets]

As you have no assets, you must pass "assets" as [] when spawning an agent.

Ensure you adhere to your plan:
[plan]

You should try to spawn agents to complete your task.

Only include one `thoughts`, `reasoning`, `decision`.

Respond in this exact JSON format exactly, with every field in order:
```json
{{
	"thoughts": "thoughts",
	"reasoning": "reasoning",
	"decision": {{
		"type": "decision type",
		"args": "..."
	}}
}}
```
"#, PhantomData);

#[derive(Serialize, Deserialize)]
pub struct NewThoughtInfo {
    pub response: String,
    pub assets: String
}

pub const NEW_THOUGHTS: Prompt<NewThoughtInfo> = Prompt(r#"
Your previous request gave back the response:
[response]

You may now make another decision, either `spawn_agent`, `brainstorm`, or `final_response`.
Try to use `thoughts` to think about what your previous response gave you, your long-term ideas, and where to go next.

Assets: 
[assets]

You may only provide these assets when spawning agents.

```json
{{
	"thoughts": "thoughts",
	"reasoning": "reasoning",
	"decision": {{
		"type": "decision type",
		"args": "..."
	}}
}}  
```
"#, PhantomData);