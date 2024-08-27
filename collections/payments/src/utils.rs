pub fn format_pubkey(text: &String, length: usize) -> String {
    if text.len() <= length {
        return text.to_string();
    }

    let half_length = length / 2;
    let ellipsis = "...";

    let start = &text[..half_length];
    let end = &text[text.len() - half_length..];

    format!("{}{}{}", start, ellipsis, end)
}
