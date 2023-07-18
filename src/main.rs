use std::{error::Error, fs};
use colored::Colorize;
pub use smartgpt::*;


fn main() -> Result<(), Box<dyn Error>> {
    let yaml_str = match fs::read_to_string("config.yml") {
        Ok(config) => config,
        Err(_) => {
            println!("{}", "Could not find 'config.yml'.".red());
            println!("Generating new config.yml...");

            fs::write("config.yml", DEFAULT_CONFIG)?;

            fs::read_to_string("config.yml")?
        }
    };


    let config = match config_from_yaml(&yaml_str) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            return Err(err);
        }
    };

    Ok(run(config)?)

}