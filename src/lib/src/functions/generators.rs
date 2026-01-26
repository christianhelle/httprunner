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
            "Aaron", "Abigail", "Adrian", "Alexander", "Alice", "Amy", "Andrew", "Angelica", "Anthony", "Ava",
            "Axel", "Ayden", "Bella", "Benjamin", "Brandi", "Brandon", "Brenda", "Brendan", "Brennan", "Brice",
            "Brianna", "Bridget", "Bridgette", "Britney", "Brooks", "Bruce", "Bruno", "Bryan", "Bryce", "Bryson",
            "Bud", "Buddy", "Caden", "Caldwell", "Caleb", "Camila", "Cameron", "Campbell", "Candace", "Candice",
            "Candy", "Cannon", "Canton", "Carmelita", "Carmen", "Carmichael", "Carol", "Carole", "Caroline", "Carolyn",
            "Carrie", "Carroll", "Carson", "Carter", "Carver", "Casandra", "Casey", "Cassandra", "Cassidy", "Cassie",
            "Castillo", "Casual", "Catalina", "Catarina", "Catherine", "Cathleen", "Cathrine", "Cathryn", "Cathy", "Catriel",
            "Catrina", "Catriona", "Cecelia", "Cecil", "Cecilia", "Cedric", "Celena", "Celesta", "Celeste", "Celia",
            "Celina", "Celine", "Celsa", "Celt", "Cena", "Cerise", "Cerys", "Cesar", "Chad", "Chadwick",
            "Chai", "Chakra", "Chalina", "Champ", "Chance", "Chandler", "Chandra", "Chandrika", "Chanel", "Chanelle",
            "Chang", "Chaney", "Channa", "Channing", "Charlie", "Connor", "Dakota", "Daphne", "David", "Diana",
            "Dina", "Earl", "Ebony", "Eddie", "Edgar", "Edison", "Edith", "Edmund", "Eduardo", "Edward",
            "Edwin", "Eileen", "Elaina", "Elaine", "Elbert", "Eldon", "Eleanor", "Eleazar", "Electra", "Elena",
            "Eleni", "Eleonora", "Eleonore", "Elesa", "Eleta", "Elfreda", "Elias", "Elida", "Elidah", "Elide",
            "Elie", "Eliel", "Eligio", "Elihu", "Elijah", "Elinor", "Elinore", "Eliot", "Elis", "Elisa",
            "Elisabeth", "Elisabet", "Elise", "Elisha", "Elisheba", "Elissa", "Elizabet", "Elizabeth", "Elizebeth", "Ella",
            "Elladine", "Elle", "Ellena", "Ellery", "Ellie", "Elliot", "Elliott", "Ellis", "Ellison", "Ellsworth",
            "Elma", "Elmara", "Elmer", "Elmira", "Elmyra", "Eloise", "Eloita", "Elon", "Elonah", "Elora",
            "Elouise", "Elsa", "Elsbert", "Else", "Elseda", "Elsha", "Elspeth", "Elston", "Elsy", "Elsworth",
            "Elta", "Elva", "Elvedin", "Elven", "Elvena", "Elverina", "Elvern", "Elverta", "Elvester", "Elvet",
            "Elvia", "Elvid", "Elvie", "Elvina", "Elvine", "Elvis", "Elvy", "Elwen", "Elwood", "Elwyn",
            "Elydia", "Elysee", "Elysia", "Elysse", "Emaid", "Emaily", "Emala", "Emalia", "Emaline", "Emam",
            "Eman", "Emanda", "Emaree", "Emari", "Emari", "Emaria", "Emariee", "Emario", "Emaris", "Emary",
            "Emasree", "Emasyl", "Ematen", "Ematine", "Emaud", "Emax", "Emazine", "Ember", "Emberly", "Embla",
            "Emblyn", "Emelda", "Emelia", "Emelina", "Emelie", "Emelina", "Emelinda", "Emelio", "Emeline", "Emelita",
            "Emeline", "Emelia", "Emelie", "Emelia", "Emelina", "Emelio", "Emeline", "Emelita", "Emelya", "Emelyane",
            "Emelyn", "Emelyne", "Emend", "Emer", "Emerald", "Emerge", "Emerson", "Emery", "Emese", "Emesta",
            "Emette", "Emetus", "Emeuta", "Emeutha", "Emeutus", "Emid", "Emida", "Emidia", "Emil", "Emile",
            "Emilia", "Emilian", "Emiliana", "Emiliano", "Emilie", "Emilina", "Emiline", "Emilinia", "Emily", "Emilynn",
            "Eminence", "Eminent", "Emir", "Emira", "Emirate", "Emirene", "Emiretta", "Emiritus", "Emiritus", "Emirsino",
            "Emisco", "Emison", "Emita", "Emitress", "Emitrice", "Emitus", "Emitya", "Emity", "Emixth", "Emizthia",
            "Emizthian", "Emizthiel", "Emizthine", "Emizthis", "Emizthus", "Emjad", "Emjay", "Emjean", "Emjee", "Emjena",
            "Emjene", "Emjenee", "Emjey", "Emjorie", "Emjoye", "Emjoyed", "Emjoy", "Emjoye", "Emjoyed", "Emjoyee",
            "Emjoyia", "Emjoyna", "Emjoyne", "Emjoyner", "Emjoys", "Emjoyu", "Emjoyue", "Emjoyus", "Emjud", "Emjude",
            "Emjudea", "Emjuder", "Emjudess", "Emjudic", "Emjudie", "Emjudin", "Emjudina", "Emjudine", "Emjudith", "Emjudits",
            "Emjudix", "Emjuds", "Emjudus", "Emjudy", "Emjudya", "Emjudye", "Emjudyn", "Emjuel", "Emjuele", "Emjuelena",
            "Emjuelia", "Emjueline", "Emjuell", "Emjuella", "Emjuelle", "Emjuelles", "Emjuelly", "Emjuelly", "Emjuellyn", "Emjuelly",
        ];
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..names.len());
        names[index].to_string()
    }
}
