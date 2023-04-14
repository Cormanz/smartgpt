pub fn apply_chunks(content: &str, chunk: usize, chunk_size: usize) -> (String, Option<String>) {
    let mut chunks = 1;
    let mut len = content.len();
    let true_len = len;
    while len > chunk_size {
        len -= chunk_size;
        chunks += 1;
    }

    let length_warning = if chunks > 1 {
        Some(format!("This file has a length of {true_len} characters. You can only read up to 5,000 characters at once. You are on chunk {chunk}, and there are {chunks} chunks. You may want to consider reading the next chunks after doing work on the first."))
    } else {
        None
    };

    let content = content.chars()
        .skip((chunk - 1) * chunk_size)
        .take(chunk_size)
        .map(|el| el.to_string())
        .collect::<Vec<_>>()
        .join("");

    (content, length_warning)
}