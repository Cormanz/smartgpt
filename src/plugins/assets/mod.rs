use std::{error::Error};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{Plugin, Command, CommandContext, CommandImpl, EmptyCycle, ScriptValue, CommandResult, CommandArgument};


#[derive(Serialize, Deserialize)]
pub struct SelfThoughts {
    pub solution: String
}

#[derive(Serialize, Deserialize)]
pub struct SaveAssets {
    pub asset: String,
    pub lines: Vec<String>
}

pub async fn save_asset(ctx: &mut CommandContext, args: ScriptValue) -> Result<ScriptValue, Box<dyn Error>> {
    let SaveAssets { asset, lines } = args.parse()?;
    ctx.assets.insert(asset, lines.join("\n"));

    Ok(ScriptValue::None)
}

pub struct NoneImpl;

#[async_trait]
impl CommandImpl for NoneImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(save_asset(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub fn create_assets() -> Plugin {
    Plugin {
        name: "Assets".to_string(),
        dependencies: vec![],
        cycle: Box::new(EmptyCycle),
        commands: vec![
            Command {
                name: "save_asset".to_string(),
                purpose: "Save an asset.".to_string(),
                args: vec![
                    CommandArgument::new("asset", r#""asset_name""#),
                    CommandArgument::new("lines", r#"[ "line 1", "line 2" ]"#)
                ],
                run: Box::new(NoneImpl)
            }
        ]
    }
}