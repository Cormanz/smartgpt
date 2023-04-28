use std::{sync::{Mutex, Arc}, error::Error};

use crate::{ScriptValue, ProgramInfo, Command, CommandContext, Expression, GPTRunError};
use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua, Error as LuaError};

pub async fn run_command(
    out: &mut String,
    name: String, command: Command, 
    context: &mut CommandContext, args: Vec<ScriptValue>
) -> Result<ScriptValue, Box<dyn Error>> {
    let result = command.run.invoke(context, args.clone()).await?;

    let args: Vec<Expression> = args.iter().map(|el| el.clone().into()).collect();
    let expr = Expression::FunctionCall(name.clone(), args);

    let json = serde_yaml::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as YAML.".to_string()))?;

    let text = format!("Command {:?} was successful and returned:\n{}", expr, json);
    out.push_str(&text);
    println!("{}", text);

    Ok(result)
}

pub fn run_script(program: &mut ProgramInfo, code: &str, lua: &Lua) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, .. 
    } = program;

    let out_mutex = Arc::new(Mutex::new(String::new()));

    for plugin in plugins {
        for command in &plugin.commands {
            let name = command.name.clone();
            let command = command.box_clone();
            let lua_context_mutex = context.clone();
            let lua_out_mutex = out_mutex.clone();
            let f = lua.create_function(move |lua, args: Variadic<_>| -> LuaResult<Value> {
                let args: Vec<ScriptValue> = args.iter()
                    .map(|el: &Value| el.clone())
                    .map(|el| ScriptValue::from_lua(el, lua))
                    .flat_map(|el| {
                        if let Ok(el) = el {
                            vec![ el ]
                        } else {
                            vec![]
                        }
                    })
                    .collect();
                
                let name = command.name.clone();
                let mut context = lua_context_mutex.lock().unwrap();
                let mut out= lua_out_mutex.lock().unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    run_command(&mut out, name.clone(), command.box_clone(), &mut context, args).await
                }).map_err(|el| LuaError::RuntimeError(
                    format!("{:?}", el)
                ))?;
                
                let result = result.to_lua(&lua)?;

                Ok(result)
            })?;
            lua.globals().set(name, f)?;
            
        }
    }

    let _ = lua.load(code).exec()?;

    let out = out_mutex.lock().unwrap();
    Ok(out.clone())
}