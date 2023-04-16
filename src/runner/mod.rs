mod parse;
mod run;
mod query;
mod convert;

use std::{error::Error, collections::HashMap};

pub use parse::*;
pub use run::*;
pub use convert::*;
pub use query::*;

use serde_json::Value;

pub async fn test_runner() -> Result<(), Box<dyn Error>> {
    let code = r#"
news_results = news_search('biology news')
for article in news_results['articles']:
    print('Title:', article['title'])
    print('Author:', article['author'])
    print('Description:', article['description'])
    print('URL:', article['url'])
    print()"#;

    let program = parse_gptscript(code)?;
    /*let mut ctx = ScriptContext {
        variables: HashMap::new()
    };
    run_body(&mut ctx, program).await?;*/
    
    let json = r#"null"#;

    let data: ScriptValue = serde_json::from_str(json)?;
    println!("{:?}", data);

    Ok(())
}