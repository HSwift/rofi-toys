pub fn make_table_column(col_text: String, max_length: usize) -> String {
    fn is_chinese_character(c: char) -> bool {
        (0x80..=0x9FFF).contains(&(c as u32))
    }
    let total_chars_count: usize = col_text
        .chars()
        .map(|c| if is_chinese_character(c) { 2 } else { 1 })
        .sum();
    if total_chars_count > max_length {
        let mut result = col_text.chars().take(max_length - 1).collect::<String>();
        result.push_str("…");
        result
    } else {
        let mut result = col_text.clone();
        result.push_str(&" ".repeat(max_length - total_chars_count));
        result
    }
}
