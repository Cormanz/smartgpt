use std::{error::Error};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{Plugin, Tool, CommandContext, CommandImpl, EmptyCycle, ScriptValue, CommandResult, ToolArgument, ToolType};

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

pub struct SaveAssetImpl;

#[async_trait]
impl CommandImpl for SaveAssetImpl {
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
        tools: vec![
            Tool {
                name: "save_asset".to_string(),
                purpose: "Save an asset.".to_string(),
                args: vec![
                    ToolArgument::new("asset", r#""asset_name""#),
                    ToolArgument::new("lines", r#"[ "line 1", "line 2" ]"#)
                ],
                run: Box::new(SaveAssetImpl),
                tool_type: ToolType::Action { needs_permission: false }
            }
        ]
    }
}