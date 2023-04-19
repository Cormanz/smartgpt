use std::error::Error;

use async_openai::types::{CreateEmbeddingRequest, EmbeddingInput};
use tokenizers::tokenizer::{Tokenizer};
use tokenizers::models::bpe::BPE;

use async_openai::Client;

use crate::LLM;

fn pad<T: Copy>(vec: &mut Vec<T>, length: usize, pad: T) {
    let current_len = vec.len();
    if current_len >= length {
        return;
    }
    let num_to_add = length - current_len;
    vec.extend(std::iter::repeat(pad).take(num_to_add));
}

pub fn get_embed(llm: &LLM, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
    let embedding = llm.model.get_base_embed(text)?;

    let mut data = embedding.iter()
        .rev()
        .take(1000)
        .rev()
        .map(|&el| el)
        .collect();

    pad(&mut data, 1000, 0.);

    Ok(data)
}

pub fn tokenize(tokenizer: &Tokenizer, text: &str) -> Vec<u32>{
    // Encode the string using the tokenizer
    let encoding = tokenizer.encode(text, true).unwrap();

    // Convert the tokens to f32 values
    encoding.get_ids().into()
}

pub fn create_tokenizer() -> Tokenizer {
    let bpe = BPE::from_file("resources/vocab.json", "resources/merges.txt")
        .build()
        .unwrap();

    Tokenizer::new(bpe)
}