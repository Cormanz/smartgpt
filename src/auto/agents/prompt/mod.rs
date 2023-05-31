use std::{collections::HashMap, error::Error, marker::PhantomData};
use serde::Serialize;

mod adept;
pub use adept::*;

pub struct Prompt<'a, T : Serialize>(pub &'a str, pub PhantomData<T>);

impl<'a, T : Serialize> Prompt<'a, T> {
    pub fn fill(&self, data: T) -> Result<String, Box<dyn Error>> {
        let items: HashMap<String, String> = serde_json::from_value(serde_json::to_value(data)?)?;
        let mut out = self.0.trim().to_string();

        for (key, value) in items {
            out = out.replace(&format!("[{key}]"), &value);
        }

        Ok(out)
    }
}