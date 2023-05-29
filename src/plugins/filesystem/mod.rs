use std::{error::Error, fmt::Display, fs::OpenOptions, path::Path};

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::{Plugin, Tool, CommandContext, CommandImpl, PluginCycle, PluginData, ScriptValue, ToolArgument, CommandResult, ToolType};
use std::{fs, io::Write};

#[derive(Debug, Clone)]
pub struct FilesNoArgError<'a>(&'a str, &'a str);

impl<'a> Display for FilesNoArgError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the '{}' tool did not receive the '{}' argument.", self.0, self.1)
    }
}

impl<'a> Error for FilesNoArgError<'a> {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileWriteArgs {
    pub name: String,
    pub lines: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileReadArgs {
    pub name: String
}

pub async fn file_write(_ctx: &mut CommandContext, args: ScriptValue, append: bool) -> Result<ScriptValue, Box<dyn Error>> {
    let _tool_name = if append { "file_append" } else { "file_write" };
    let args: FileWriteArgs = args.parse()?;

    let path = args.name.strip_prefix("./").unwrap_or(&args.name).to_string();
    let path = path.strip_prefix("files/").unwrap_or(&path).to_string();

    if !Path::new("./files/").exists() {
            fs::create_dir("./files/")?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(append)
        .create(true)
        .open(format!("./files/{path}"))?;
    writeln!(file, "{}", args.lines.join("\n"))?;

    Ok(ScriptValue::None)
}

pub async fn file_list(_ctx: &mut CommandContext, _args: ScriptValue) -> Result<ScriptValue, Box<dyn Error>> {
    let files = fs::read_dir("./files/")?;
    let files = files
        .map(|el| el.map(|el| el.path().display().to_string()))
        .filter(|el| el.is_ok())
        .map(|el| el.unwrap())
        .collect::<Vec<_>>();

    Ok(ScriptValue::List(files.iter().map(|el| el.clone().into()).collect()))
}

pub async fn file_read(_ctx: &mut CommandContext, args: ScriptValue) -> Result<ScriptValue, Box<dyn Error>> {
    let FileReadArgs { name: path } = args.parse()?;
    let path = path.strip_prefix("./").unwrap_or(&path).to_string();
    let path = path.strip_prefix("files/").unwrap_or(&path).to_string();

    let content = fs::read_to_string(format!("files/{path}"))?;

    Ok(content.to_string().into())
}

pub struct FileWriteImpl;

#[async_trait]
impl CommandImpl for FileWriteImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(file_write(ctx, args, false).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileAppendImpl;

#[async_trait]
impl CommandImpl for FileAppendImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(file_write(ctx, args, true).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}


pub struct FileListImpl;

#[async_trait]
impl CommandImpl for FileListImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(file_list(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileReadImpl;

#[async_trait]
impl CommandImpl for FileReadImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: ScriptValue) -> Result<CommandResult, Box<dyn Error>> {
        Ok(CommandResult::ScriptValue(file_read(ctx, args).await?))
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileCycle;

#[async_trait]
impl PluginCycle for FileCycle {
    async fn create_context(&self, _context: &mut CommandContext, _previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        let files = fs::read_dir("files")?;
        let files = files
            .map(|el| el.map(|el| el.path().display().to_string()))
            .filter(|el| el.is_ok())
            .map(|el| el.unwrap())
            .collect::<Vec<_>>();

        Ok(Some(if files.len() == 0 {
            "Files: No saved files.".to_string()
        } else {
            format!("Files: {} (Consider reading these.)", files.join(", "))
        }))
    }

    fn create_data(&self, _value: Value) -> Option<Box<dyn PluginData>> {
        None
    }
}

pub fn create_filesystem() -> Plugin {
    Plugin {
        name: "File System".to_string(),
        dependencies: vec![],
        cycle: Box::new(FileCycle),
        tools: vec![
            Tool {
                name: "file_write".to_string(),
                purpose: "Override a file with content. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![
                    ToolArgument::new("name", r#""name""#),
                    ToolArgument::new("lines", r#"[ "line 1", "line 2" ]"#)
                ],
                run: Box::new(FileWriteImpl),
                tool_type: ToolType::Resource
            },
            Tool {
                name: "file_append".to_string(),
                purpose: "Add content to an existing file. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![
                    ToolArgument::new("name", r#""name""#),
                    ToolArgument::new("lines", r#"[ "line 1", "line 2" ]"#)
                ],
                run: Box::new(FileAppendImpl),
                tool_type: ToolType::Resource
            },
            Tool {
                name: "file_list".to_string(),
                purpose: "List all of your files.".to_string(),
                args: vec![],
                run: Box::new(FileListImpl),
                tool_type: ToolType::Resource
            },
            Tool {
                name: "file_read".to_string(),
                purpose: "Read a file.".to_string(),
                args: vec![
                    ToolArgument::new("name", r#""name""#),
                ],
                run: Box::new(FileReadImpl),
                tool_type: ToolType::Resource
            }
        ]
    }
}
