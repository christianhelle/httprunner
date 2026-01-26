use crate::functions::substitution::FunctionSubstitutor;
use std::result::Result;

pub struct GuidSubstitutor {}
impl FunctionSubstitutor for GuidSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bguid\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        format!(
            "{:08x}{:04x}{:04x}{:04x}{:012x}",
            u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_be_bytes([bytes[4], bytes[5]]),
            (u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff) | 0x4000,
            (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000,
            u64::from_be_bytes([
                0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
            ]) & 0xffffffffffff
        )
    }
}

pub struct StringSubstitutor {}
impl FunctionSubstitutor for StringSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bstring\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        use rand::distributions::Alphanumeric;

        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect()
    }
}

pub struct NumberSubstitutor {}
impl FunctionSubstitutor for NumberSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bnumber\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;

        rand::thread_rng().gen_range(0..=100).to_string()
    }
}

pub struct Base64EncodeSubstitutor {}
impl FunctionSubstitutor for Base64EncodeSubstitutor {
    fn get_regex(&self) -> &str {
        r"(?!)"
    }

    fn generate(&self) -> String {
        String::new()
    }

    fn replace(&self, input: &str) -> Result<String, regex::Error> {
        use base64::Engine;
        use base64::engine::general_purpose;
        use regex::RegexBuilder;

        let re = RegexBuilder::new(r"\bbase64_encode\(\s*'((?:[^'\\]|\\.)*)'\s*\)")
            .case_insensitive(true)
            .build()?;
        Ok(re
            .replace_all(input, |caps: &regex::Captures| {
                let to_encode = &caps[1];
                general_purpose::STANDARD.encode(to_encode)
            })
            .to_string())
    }
}

pub struct NameSubstitutor {}
impl FunctionSubstitutor for NameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bname\(\)"
    }

    fn generate(&self) -> String {
        let names = vec![
            // A
            "Aaron", "Abigail", "Adrian", "Alexander", "Alice", "Amy", "Andrew", "Angelica", "Anthony", "Ava",
            // B
            "Benjamin", "Bella", "Bob", "Brandon", "Brenda", "Brendan", "Bridget", "Bruce", "Bryan", "Bradley",
            // C
            "Caleb", "Cameron", "Camila", "Candace", "Carter", "Casey", "Catherine", "Charlie", "Christopher", "Connor",
            // D
            "Dakota", "Daniel", "David", "Danielle", "Daphne", "Deborah", "Derek", "Diana", "Donna", "Dorothy",
            // E
            "Ethan", "Eleanor", "Elijah", "Elizabeth", "Emily", "Emma", "Eric", "Evelyn", "Edward", "Elliott",
            // F
            "Fiona", "Felix", "Frank", "Francesca", "Faye", "Finn", "Felicity", "Frederick", "Freya", "Francis",
            // G
            "George", "Grace", "Gabriella", "Gregory", "Grayson", "Gwendolyn", "Gavin", "Garrett", "Gary", "Gail",
            // H
            "Hannah", "Henry", "Harrison", "Heidi", "Helen", "Hector", "Hazel", "Howard", "Harold", "Henrietta",
            // I
            "Ian", "Isaac", "Isabella", "Iris", "Ivan", "Irene", "Ingrid", "Igor", "Imogen", "Isadora",
            // J
            "Jack", "James", "Jayden", "Jessica", "Jerome", "Janet", "Jasmine", "Jared", "Jennifer", "Josephine",
            // K
            "Kevin", "Kathryn", "Keisha", "Kyle", "Keith", "Kimberley", "Kirk", "Kayla", "Kenneth", "Katherine",
            // L
            "Lily", "Liam", "Landon", "Lena", "Leo", "Lisa", "Lionel", "Lucia", "Logan", "Lydia",
            // M
            "Michael", "Melanie", "Marcus", "Melissa", "Martin", "Marcella", "Matthew", "Monica", "Magnolia", "Michelle",
            // N
            "Natalie", "Nathan", "Nathaniel", "Nancy", "Nicholas", "Natasha", "Nelson", "Nicole", "Noah", "Norman",
            // O
            "Oliver", "Olivia", "Oscar", "Opal", "Owen", "Ophelia", "Otto", "Octavia", "Orson", "Olive",
            // P
            "Patricia", "Patrick", "Parker", "Paisley", "Peter", "Pamela", "Paul", "Phillip", "Phoebe", "Patrice",
            // Q
            "Quentin", "Quinn", "Quinton", "Quincy", "Quinlan", "Quinley", "Quill", "Queenie", "Quest", "Quinby",
            // R
            "Rebecca", "Rachel", "Rosa", "Rhonda", "Richard", "Robert", "Raymond", "Randall", "Rita", "Ryan",
            // S
            "Samuel", "Stephen", "Sebastian", "Stella", "Sophia", "Susan", "Steven", "Sarah", "Scott", "Sandra",
            // T
            "Thomas", "Tanya", "Teresa", "Tabitha", "Timothy", "Theodore", "Theresa", "Tina", "Tyler", "Terrence",
            // U
            "Uriel", "Ulysses", "Ulrich", "Upton", "Uma", "Urban", "Unique", "Udo", "Usher", "Unity",
            // V
            "Victoria", "Valerie", "Vanessa", "Vincent", "Victor", "Violet", "Vernon", "Vivian", "Vaughn", "Valencia",
            // W
            "William", "Warren", "Wesley", "Wyatt", "Walter", "Winona", "Wayne", "Willow", "Winston", "Wanda",
            // X
            "Xavier", "Xena", "Ximena", "Xiomara", "Xander", "Xanthe", "Xavi", "Xenia", "Xiaowen", "Xyla",
            // Y
            "Yvonne", "Yolanda", "Yasmine", "Yancy", "Yara", "York", "Yusuf", "Yannick", "Yosef", "Yuri",
            // Z
            "Zachary", "Zoe", "Zelda", "Zara", "Zeke", "Zena", "Ziggy", "Zola", "Zuri", "Zane",
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..names.len());
        names[index].to_string()
    }
}
