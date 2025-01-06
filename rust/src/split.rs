pub fn split_into_sentences(input: &str) -> Vec<String> {
    let chars = &['.', '!', '?'];
    let strings = &[".", "!", "?"];

    input
        .lines()
        .flat_map(|line| {
            line.split_inclusive(chars)
                .map(str::trim)
                .filter(|sentence| !(sentence.is_empty() || strings.contains(sentence)))
                .map(String::from)
                .map(|x| x.to_lowercase()) // less hallucination
        })
        .collect()
}
