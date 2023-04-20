use std::{error::Error, fmt::Display, collections::HashMap, process::CommandArgs};

use async_openai::{types::{CreateChatCompletionRequest, CreateChatCompletionResponse, ChatCompletionRequestMessage, Role}, error::OpenAIError, Client};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CommandContext, CommandImpl, Plugin, EmptyCycle, Command, CommandNoArgError, PluginData, PluginDataNoInvoke, invoke, PluginCycle, ScriptValue, CommandArgument};

const CHAT_GPT_PROMPT: &str = r#"You are ChatGPT, a large language model trained by OpenAI, based on the GPT-3.5 architecture. As an assistant, your purpose is to provide helpful and informative responses to a wide variety of questions and topics, while also engaging in natural and friendly conversation with users.

As ChatGPT, you must always prioritize safety and appropriate behavior in all interactions. This means that you are programmed to avoid any content that could be harmful or offensive, and to always maintain a respectful and polite tone."#;

pub struct ChatGPTData {
    pub client: Client,
    pub memory: Vec<ChatCompletionRequestMessage>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatGPTPluginConfig {
    #[serde(rename = "api key")] pub api_key: String
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ChatGPTRole {
    Assistant,
    System,
    User
}

impl From<ChatGPTRole> for Role {
    fn from(value: ChatGPTRole) -> Self {
        match value {
            ChatGPTRole::Assistant => Role::Assistant,
            ChatGPTRole::System => Role::System,
            ChatGPTRole::User => Role::User
        }
    }
}

impl From<Role> for ChatGPTRole {
    fn from(value: Role) -> Self {
        match value {
            Role::Assistant => ChatGPTRole::Assistant,
            Role::System => ChatGPTRole::System,
            Role::User => ChatGPTRole::User
        }
    }
}

impl From<ChatGPTMessage> for ChatCompletionRequestMessage {
    fn from(value: ChatGPTMessage) -> Self {
        ChatCompletionRequestMessage {
            role: value.role.into(),
            content: value.content,
            name: None
        }
    }
}

impl From<ChatCompletionRequestMessage> for ChatGPTMessage {
    fn from(value: ChatCompletionRequestMessage) -> Self {
        ChatGPTMessage {
            role: value.role.into(),
            content: value.content
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatGPTMessage {
    role: ChatGPTRole,
    content: String
}

#[async_trait]
impl PluginData for ChatGPTData {
    async fn apply(&mut self, name: &str, value: Value) -> Result<Value, Box<dyn Error>> {
        match name {
            "len" => {
                Ok(self.memory.len().into())
            }
            "push" => {
                let ChatGPTMessage { role, content } = serde_json::from_value(value)?;

                self.memory.push(
                    ChatCompletionRequestMessage {
                        role: role.into(),
                        content,
                        name: None
                    }
                );

                Ok(true.into())
            }
            "clear" => {
                self.memory.clear();
                Ok(true.into())
            }
            "respond" => {
                let mut request = CreateChatCompletionRequest::default();

                let messages: Vec<ChatCompletionRequestMessage> = self.memory
                    .iter()
                    .map(|el| el.clone().into())
                    .collect::<Vec<_>>();
            
                request.model = "gpt-3.5-turbo".to_string();
                request.messages = messages;

                let response: CreateChatCompletionResponse = self.client
                    .chat()      // Get the API "group" (completions, images, etc.) from the client
                    .create(request.clone()).await?;

                Ok(response.choices[0].message.content.clone().into())
            }
            "get" => {
                let gpt_messages: Vec<ChatGPTMessage> = self.memory.iter()
                    .map(|el| el.clone().into())
                    .collect::<Vec<_>>();
                let gpt_messages: Vec<Value> = gpt_messages.iter()
                    .map(|el| serde_json::to_value(el).unwrap())
                    .collect::<Vec<_>>();
                Ok(gpt_messages.into())
            }
            _ => {
                Err(Box::new(PluginDataNoInvoke("ChatGPT".to_string(), name.to_string())))
            }
        }
    }
}

pub async fn ask_chatgpt(context: &mut CommandContext, query: &str) -> Result<String, Box<dyn Error>> {
    let chatgpt_info = context.plugin_data.get_data("ChatGPT")?;

    let len = invoke::<usize>(chatgpt_info, "len", true).await?;

    if len == 0 {
        invoke::<bool>(chatgpt_info, "push", ChatGPTMessage {
            role: ChatGPTRole::System,
            content: CHAT_GPT_PROMPT.to_string()
        }).await?;
    }

    invoke::<bool>(chatgpt_info, "push", ChatGPTMessage {
        role: ChatGPTRole::User,
        content: query.to_string()
    }).await?;

    let content = invoke::<String>(chatgpt_info, "respond", true).await?;
    
    invoke::<bool>(chatgpt_info, "push", ChatGPTMessage {
        role: ChatGPTRole::Assistant,
        content: content.clone()
    }).await?;

    Ok(content.clone())
}

pub async fn chatgpt(ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let prompt: String = args.get(0).ok_or(CommandNoArgError("ask-chatgpt", "prompt"))?.clone().try_into()?;
    let response = ask_chatgpt(ctx, &prompt).await?;
    
    Ok(response.into())
}

pub async fn reset_chatgpt(ctx: &mut CommandContext, _: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    let chatgpt_info = ctx.plugin_data.get_data("ChatGPT")?;
    invoke::<bool>(chatgpt_info, "clear", true).await?;
    
    Ok(ScriptValue::None)
}

pub struct ChatGPTImpl;

#[async_trait]
impl CommandImpl for ChatGPTImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        chatgpt(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct ResetChatGPTImpl;

#[async_trait]
impl CommandImpl for ResetChatGPTImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
        reset_chatgpt(ctx, args).await
    }

    fn box_clone(&self) -> Box<dyn CommandImpl> {
        Box::new(Self)
    }
}

pub struct ChatGPTCycle;

#[async_trait]
impl PluginCycle for ChatGPTCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(None)
    }

    fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>> {
        let config: ChatGPTPluginConfig = serde_json::from_value(value).ok()?;

        Some(Box::new(ChatGPTData {
            client: Client::new().with_api_key(config.api_key.clone()),
            memory: vec![]
        }))
    }
}

pub fn create_chatgpt() -> Plugin {
    Plugin {
        name: "ChatGPT".to_string(),
        dependencies: vec![],
        cycle: Box::new(ChatGPTCycle),
        commands: vec![
            Command {
                name: "ask_chatgpt".to_string(),
                purpose: "Ask ChatGPT to answer your prompt.".to_string(),
                args: vec![
                    CommandArgument::new("prompt", "The prompt to ask ChatGPT.", "String")
                ],
                return_type: "String".to_string(),
                run: Box::new(ChatGPTImpl)
            },
            Command {
                name: "reset_chatgpt".to_string(),
                purpose: "Reset the memory of ChatGPT.".to_string(),
                args: vec![],
                return_type: "None".to_string(),
                run: Box::new(ResetChatGPTImpl)
            }
        ]
    }
}