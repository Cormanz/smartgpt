use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::Prompt;

#[derive(Serialize, Deserialize)]
pub struct NoData;

pub const SUMMARIZE_MEMORIES: Prompt<NoData> = Prompt(r#"
Please summarize all important actions you took out.
Please also summarize all observations of information you have collected.

Be concise.

Respond in this JSON format:
```json
{{
	"actions": [
		"what tool you used and why"
	],
	"observations": [
		"what you learned"
	]
}}
```"#, PhantomData);

#[derive(Serialize, Deserialize)]
pub struct CreatePlanInfo {
    pub tools: String,
    pub task: String,
    pub observations: String,
    pub assets: String,
    pub desire: String
}

pub const CREATE_PLAN: Prompt<CreatePlanInfo> = Prompt(r#"
[tools]

You have been given these resources and actions.
You may use these resources and actions, and only these.

Here is your new task:
[task]

Here is a list of your memories:
[observations]

Here is a list of assets previously saved:
[assets]

Create a list of steps of what you need to do and which resource or action you will use.
Only use one resource or action for each step.

Your goal is to give a response with the following information:
[desire]

You should try to save that precise information through assets.

Do not specify arguments.
Do not "repeat steps".

Keep your plan at as low steps as possible.
Keep your plan as concise as possible!

After you are done planning steps, additionally plan to save one or more assets as output.

Respond in this JSON format:
```json
{{
	"thoughts": "thoughts regarding steps and assets",
	"steps": [
		{{
			"idea": "idea",
			"decision": {{
				"resource": {{
					"name": "name",
					"question": "what question does using this resource answer"
				}}
			}}
		}},
		{{
			"idea": "idea",
			"decision": {{
				"action": {{
					"name": "name",
					"purpose": "why use this action"
				}}
			}}
		}}
	],
	"assets": [
		{{
			"name": "asset_name,
			"description": "description"
		}}
	]
}}
```
"#, PhantomData);

#[derive(Serialize, Deserialize)]
pub struct NextStepInfo {
    pub step: String
}

pub const NEXT_STEP: Prompt<NextStepInfo> = Prompt(r#"
Now you will carry out the next step: 
[step]

You must carry out this step with one entire action.
Include ALL information.

Ensure you don't hallucinate; only give information that you actually have.

Assets:
No assets.

Respond in this JSON format:
```json
{{
	"thoughts": "thoughts",
	"action": {{
		"tool": "tool",
		"args": {{}}
	}}
}}
```
"#, PhantomData);

#[derive(Serialize, Deserialize)]
pub struct SaveAssetInfo {
    pub asset: String
}

pub const SAVE_ASSET: Prompt<SaveAssetInfo> = Prompt(r#"
Now, you will write this asset:

[asset]

Respond in pure plaintext format with a detailed markdown response.
Include all necessary details as the description stated, alongside any necessary sources or explanation of where you got the information.
"#, PhantomData);