use super::substitution::FunctionSubstitutor;

pub(crate) static LOREM_IPSUM_WORDS: &[&str] = &[
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "enim",
    "ad",
    "minim",
    "veniam",
    "quis",
    "nostrud",
    "exercitation",
    "ullamco",
    "laboris",
    "nisi",
    "aliquip",
    "ex",
    "ea",
    "commodo",
    "consequat",
    "duis",
    "aute",
    "irure",
    "in",
    "reprehenderit",
    "voluptate",
    "velit",
    "esse",
    "cillum",
    "fugiat",
    "nulla",
    "pariatur",
    "excepteur",
    "sint",
    "occaecat",
    "cupidatat",
    "non",
    "proident",
    "sunt",
    "culpa",
    "qui",
    "officia",
    "deserunt",
    "mollit",
    "anim",
    "id",
    "est",
    "laborum",
    "at",
    "vero",
    "eos",
    "accusamus",
    "iusto",
    "odio",
    "dignissimos",
    "ducimus",
    "blanditiis",
    "praesentium",
    "voluptatum",
    "deleniti",
    "atque",
    "corrupti",
    "quos",
    "dolores",
    "quas",
    "molestias",
    "excepturi",
    "obcaecati",
    "cupiditate",
    "provident",
    "similique",
    "mollitia",
    "maiores",
    "alias",
    "consequatur",
    "perferendis",
    "doloribus",
    "asperiores",
    "repellat",
    "temporibus",
    "quibusdam",
    "aut",
    "officiis",
    "debitis",
    "rerum",
    "necessitatibus",
    "saepe",
    "eveniet",
    "voluptates",
    "repudiandae",
    "recusandae",
    "itaque",
    "earum",
    "hic",
    "tenetur",
    "sapiente",
    "delectus",
    "aut",
    "reiciendis",
    "maiores",
    "alias",
    "consequatur",
    "aut",
    "perferendis",
    "doloribus",
    "asperiores",
    "repellat",
    "hanc",
    "egredientur",
    "totam",
    "rem",
    "aperiam",
    "eaque",
    "ipsa",
    "quae",
    "ab",
    "illo",
    "inventore",
    "veritatis",
    "quasi",
    "architecto",
    "beatae",
    "vitae",
    "dicta",
    "explicabo",
    "nemo",
    "ipsam",
    "quia",
    "voluptas",
    "aspernatur",
    "odit",
    "aut",
    "fugit",
    "sed",
    "quia",
    "consequuntur",
    "magni",
    "dolores",
    "eos",
    "qui",
    "ratione",
    "voluptatem",
    "sequi",
    "nesciunt",
    "neque",
    "porro",
    "quisquam",
    "dolorem",
    "ipsum",
    "quia",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipisci",
    "velit",
    "sed",
    "quia",
    "numquam",
    "eius",
    "modi",
    "tempora",
    "incidunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magnam",
    "aliquam",
    "quaerat",
    "voluptatem",
    "ullam",
    "corporis",
    "suscipit",
    "laboriosam",
    "nisi",
    "ut",
    "aliquid",
    "ex",
    "ea",
    "commodi",
    "consequatur",
    "quis",
    "autem",
    "vel",
    "eum",
    "iure",
    "reprehenderit",
    "qui",
    "in",
    "ea",
    "voluptate",
    "velit",
    "esse",
    "quam",
    "nihil",
    "molestiae",
    "consequatur",
    "vel",
    "illum",
    "qui",
    "dolorem",
    "eum",
    "fugiat",
    "quo",
    "voluptas",
    "nulla",
    "pariatur",
    "temporibus",
    "autem",
    "quibusdam",
    "et",
    "aut",
    "officiis",
    "debitis",
    "aut",
    "rerum",
    "necessitatibus",
    "saepe",
    "eveniet",
    "ut",
    "et",
    "voluptates",
    "repudiandae",
    "sint",
    "et",
    "molestiae",
    "non",
    "recusandae",
    "itaque",
    "earum",
    "rerum",
    "hic",
    "tenetur",
    "a",
    "sapiente",
    "delectus",
    "ut",
    "aut",
    "reiciendis",
    "voluptatibus",
    "maiores",
    "alias",
    "consequatur",
    "aut",
    "perferendis",
    "doloribus",
    "asperiores",
    "repellat",
    "nam",
    "libero",
    "tempore",
    "cum",
    "soluta",
    "nobis",
    "est",
    "eligendi",
    "optio",
    "cumque",
    "nihil",
    "impedit",
    "minus",
    "id",
    "quod",
    "maxime",
    "placeat",
    "facere",
    "possimus",
    "omnis",
    "voluptas",
    "assumenda",
    "est",
    "omnis",
    "dolor",
    "repellendus",
];

pub struct LoremIpsumSubstitutor {}
impl FunctionSubstitutor for LoremIpsumSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use regex::RegexBuilder;

        let pattern = r"\blorem_ipsum\(\s*(\d*)\s*\)";
        let regex = RegexBuilder::new(pattern).case_insensitive(true).build()?;
        Ok(regex
            .replace_all(input, |caps: &regex::Captures| {
                let val = caps[1].parse::<usize>().unwrap_or(100);
                if val < LOREM_IPSUM_WORDS.len() {
                    LOREM_IPSUM_WORDS
                        .iter()
                        .take(val)
                        .map(|w| w.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                } else {
                    let mut words = Vec::new();
                    for i in 0..val {
                        words.push(LOREM_IPSUM_WORDS[i % LOREM_IPSUM_WORDS.len()].to_string());
                    }
                    words.join(" ")
                }
            })
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lorem_ipsum() {
        let lorem_ipsum = LoremIpsumSubstitutor {};
        let input = "lorem_ipsum(5)";
        let result = lorem_ipsum.replace(input).unwrap();
        assert!(result.contains("lorem ipsum dolor sit amet"));
        assert_eq!(result.split_whitespace().count(), 5);
    }

    #[test]
    fn test_lorem_ipsum_exceeding_max_length() {
        let len = LOREM_IPSUM_WORDS.len() * 2;
        let lorem_ipsum = LoremIpsumSubstitutor {};
        let input = format!("lorem_ipsum({})", len);
        let result = lorem_ipsum.replace(&input).unwrap();
        assert!(result.contains("lorem ipsum dolor sit amet"));
        assert_eq!(result.split_whitespace().count(), len);
    }

    #[test]
    fn test_lorem_ipsum_case_insensitive() {
        let lorem_ipsum = LoremIpsumSubstitutor {};
        let input = "LOREM_IPSUM(5)";
        let result = lorem_ipsum.replace(input).unwrap();
        assert!(result.contains("lorem ipsum dolor sit amet"));
    }

    #[test]
    fn test_lorem_ipsum_empty() {
        let lorem_ipsum = LoremIpsumSubstitutor {};
        let input = "lorem_ipsum()";
        let result = lorem_ipsum.replace(input).unwrap();
        assert_eq!(result.split_whitespace().count(), 100);
    }
}
