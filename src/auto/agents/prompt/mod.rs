use std::{collections::HashMap, error::Error, marker::PhantomData};
use serde::Serialize;

mod adept;
mod methodical;

pub use adept::*;
pub use methodical::*;

pub struct Prompt<'a, T : Serialize>(pub &'a str, pub PhantomData<T>);

impl<'a, T : Serialize> Prompt<'a, T> {
    pub fn fill(&self, data: T) -> Result<String, Box<dyn Error>> {
        let mut out = self.0.trim().to_string();

        let items: HashMap<String, String> = match serde_json::from_value(serde_json::to_value(data)?) {
            Ok(items) => items,
            Err(_) => {
                return Ok(out);
            }
        };

        for (key, value) in items {
            out = out.replace(&format!("[{key}]"), &value);
        }

        Ok(out)
    }
}