use std::{collections::HashMap, error::Error, fmt::Display, fs::OpenOptions, path::Path};

use async_trait::async_trait;
use serde_json::Value;

use crate::{Plugin, Command, CommandContext, CommandImpl, PluginCycle, apply_chunks, PluginData, ScriptValue, CommandArgument};
use std::{fs, io::Write};

#[derive(Debug, Clone)]
pub struct FilesNoArgError<'a>(&'a str, &'a str);

impl<'a> Display for FilesNoArgError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the '{}' command did not receive the '{}' argument.", self.0, self.1)
    }
}

impl<'a> Error for FilesNoArgError<'a> {}

pub async fn file_write(ctx: &mut CommandContext, args: Vec<ScriptValue>, append: bool) -> Result<ScriptValue, Box<dyn Error>> {
    let command_name = if append { "file_append" } else { "file_write" };
    let path: String = args.get(0)
        .ok_or(FilesNoArgError(command_name, "path"))?
        .clone().try_into()?;

    let mut content = String::new();
    let contents = args.iter()
        .skip(1);

    for arg in contents {
        let arg_content: String = arg.clone().try_into()?;
        content.push_str(&arg_content);
    }

    if args.len() <= 1 {
        return Err(Box::new(FilesNoArgError(command_name, "content")));
    }

    let path = path.strip_prefix("./").unwrap_or(&path).to_string();
    let path = path.strip_prefix("files/").unwrap_or(&path).to_string();

    if !Path::new("./files/").exists() {
            fs::create_dir("./files/")?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(append)
        .create(true)
        .open(format!("./files/{path}"))?;
    writeln!(file, "{}", content)?;

    Ok(ScriptValue::None)
}

pub async fn file_list(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let files = fs::read_dir("./files/")?;
    let files = files
        .map(|el| el.map(|el| el.path().display().to_string()))
        .filter(|el| el.is_ok())
        .map(|el| el.unwrap())
        .collect::<Vec<_>>();

    Ok(ScriptValue::List(files.iter().map(|el| el.clone().into()).collect()))
}

pub async fn file_read(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let path: String = args.get(0).ok_or(FilesNoArgError("file_read", "path"))?.clone().try_into()?;
    let path = path.strip_prefix("./").unwrap_or(&path).to_string();
    let path = path.strip_prefix("files/").unwrap_or(&path).to_string();

    let content = fs::read_to_string(format!("files/{path}"))?;

    Ok(content.to_string().into())
}

pub struct FileWriteImpl;

#[async_trait]
impl CommandImpl for FileWriteImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        file_write(ctx, args, false).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileAppendImpl;

#[async_trait]
impl CommandImpl for FileAppendImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        file_write(ctx, args, true).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}


pub struct FileListImpl;

#[async_trait]
impl CommandImpl for FileListImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        file_list(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileReadImpl;

#[async_trait]
impl CommandImpl for FileReadImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        file_read(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct FileCycle;

#[async_trait]
impl PluginCycle for FileCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
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

    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        None
    }
}

pub fn create_filesystem() -> Plugin {
    Plugin {
        name: "File System".to_string(),
        dependencies: vec![],
        cycle: Box::new(FileCycle),
        commands: vec![
            Command {
                name: "file_write".to_string(),
                purpose: "Override a file with content. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![
                    CommandArgument::new("path", "The path of the file that is being written to.", "String"),
                    CommandArgument::new("...contents", "The content to be added to the file. You can use as many arguments for content as you like.", "String")
                ],
                return_type: "None".to_string(),
                run: Box::new(FileWriteImpl)
            },
            Command {
                name: "file_append".to_string(),
                purpose: "Add content to an existing file. Just use a raw file name, no folders or extensions, like 'cheese salad'.".to_string(),
                args: vec![
                    CommandArgument::new("path", "The path of the file that is being written to.", "String"),
                    CommandArgument::new("...contents", "The content to be added to the file. You can use as many arguments for content as you like.", "String")
                ],
                return_type: "None".to_string(),
                run: Box::new(FileAppendImpl)
            },
            Command {
                name: "file_list".to_string(),
                purpose: "List all of your files.".to_string(),
                args: vec![],
                return_type: "String[]".to_string(),
                run: Box::new(FileListImpl)
            },
            Command {
                name: "file_read".to_string(),
                purpose: "Read a file.".to_string(),
                args: vec![
                    CommandArgument::new("path", "The path of the file that is read.", "String")
                ],
                return_type: "String".to_string(),
                run: Box::new(FileReadImpl)
            }
        ]
    }
}
