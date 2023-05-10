pub fn find_text_between_braces(input: &str) -> Option<String> {
    let start_index = match input.find('{') {
        Some(index) => index,
        None => return None, // No opening brace found
    };

    let end_index = match input.rfind('}') {
        Some(index) => index,
        None => return None, // No closing brace found
    };

    if start_index >= end_index {
        return None; // Closing brace comes before opening brace or no text in between
    }

    Some(format!("{}{}{}", "{", &input[start_index + 1..end_index], "}"))
}