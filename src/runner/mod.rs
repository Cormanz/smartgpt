mod parse;
mod scriptvalue;
mod convert;

use std::{error::Error, collections::HashMap, time::Duration, fs, sync::{Mutex, Arc}};

use colored::Colorize;
pub use parse::*;
pub use scriptvalue::*;
pub use convert::*;

use tokio::{time::sleep, runtime::{Handle, Runtime}};

use mlua::{
    AnyUserData, ExternalResult, Lua, Result as LuaResult, 
    UserData, UserDataMethods, Value, FromLua, Error as LuaError, Variadic, ToLua
};

use crate::{load_config, ProgramInfo, Command, Context, CommandContext, browse_url};

pub fn test_runner() -> Result<(), Box<dyn Error>> {
    let url = "https://codilime.com/blog/why-is-rust-programming-language-so-popular/#:~:text=The%20Rust%20programming%20language%20has,to%20build%20secure%20operating%20systems.";
    
    let config = fs::read_to_string("config.yml")?;
    let mut program = load_config(&config)?;

    Ok(())
}