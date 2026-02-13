use super::substitution::FunctionSubstitutor;

pub(crate) static FIRST_NAMES: &[&str] = &[
    // A
    "Aaron",
    "Abigail",
    "Adrian",
    "Alexander",
    "Alice",
    "Amy",
    "Andrew",
    "Angelica",
    "Anthony",
    "Ava",
    // B
    "Benjamin",
    "Bella",
    "Bob",
    "Brandon",
    "Brenda",
    "Brendan",
    "Bridget",
    "Bruce",
    "Bryan",
    "Bradley",
    // C
    "Caleb",
    "Cameron",
    "Camila",
    "Candace",
    "Carter",
    "Casey",
    "Catherine",
    "Charlie",
    "Christopher",
    "Connor",
    // D
    "Dakota",
    "Daniel",
    "David",
    "Danielle",
    "Daphne",
    "Deborah",
    "Derek",
    "Diana",
    "Donna",
    "Dorothy",
    // E
    "Ethan",
    "Eleanor",
    "Elijah",
    "Elizabeth",
    "Emily",
    "Emma",
    "Eric",
    "Evelyn",
    "Edward",
    "Elliott",
    // F
    "Fiona",
    "Felix",
    "Frank",
    "Francesca",
    "Faye",
    "Finn",
    "Felicity",
    "Frederick",
    "Freya",
    "Francis",
    // G
    "George",
    "Grace",
    "Gabriella",
    "Gregory",
    "Grayson",
    "Gwendolyn",
    "Gavin",
    "Garrett",
    "Gary",
    "Gail",
    // H
    "Hannah",
    "Henry",
    "Harrison",
    "Heidi",
    "Helen",
    "Hector",
    "Hazel",
    "Howard",
    "Harold",
    "Henrietta",
    // I
    "Ian",
    "Isaac",
    "Isabella",
    "Iris",
    "Ivan",
    "Irene",
    "Ingrid",
    "Igor",
    "Imogen",
    "Isadora",
    // J
    "Jack",
    "James",
    "Jayden",
    "Jessica",
    "Jerome",
    "Janet",
    "Jasmine",
    "Jared",
    "Jennifer",
    "Josephine",
    // K
    "Kevin",
    "Kathryn",
    "Keisha",
    "Kyle",
    "Keith",
    "Kimberley",
    "Kirk",
    "Kayla",
    "Kenneth",
    "Katherine",
    // L
    "Lily",
    "Liam",
    "Landon",
    "Lena",
    "Leo",
    "Lisa",
    "Lionel",
    "Lucia",
    "Logan",
    "Lydia",
    // M
    "Michael",
    "Melanie",
    "Marcus",
    "Melissa",
    "Martin",
    "Marcella",
    "Matthew",
    "Monica",
    "Magnolia",
    "Michelle",
    // N
    "Natalie",
    "Nathan",
    "Nathaniel",
    "Nancy",
    "Nicholas",
    "Natasha",
    "Nelson",
    "Nicole",
    "Noah",
    "Norman",
    // O
    "Oliver",
    "Olivia",
    "Oscar",
    "Opal",
    "Owen",
    "Ophelia",
    "Otto",
    "Octavia",
    "Orson",
    "Olive",
    // P
    "Patricia",
    "Patrick",
    "Parker",
    "Paisley",
    "Peter",
    "Pamela",
    "Paul",
    "Phillip",
    "Phoebe",
    "Patrice",
    // Q
    "Quentin",
    "Quinn",
    "Quinton",
    "Quincy",
    "Quinlan",
    "Quinley",
    "Quill",
    "Queenie",
    "Quest",
    "Quinby",
    // R
    "Rebecca",
    "Rachel",
    "Rosa",
    "Rhonda",
    "Richard",
    "Robert",
    "Raymond",
    "Randall",
    "Rita",
    "Ryan",
    // S
    "Samuel",
    "Stephen",
    "Sebastian",
    "Stella",
    "Sophia",
    "Susan",
    "Steven",
    "Sarah",
    "Scott",
    "Sandra",
    // T
    "Thomas",
    "Tanya",
    "Teresa",
    "Tabitha",
    "Timothy",
    "Theodore",
    "Theresa",
    "Tina",
    "Tyler",
    "Terrence",
    // U
    "Uriel",
    "Ulysses",
    "Ulrich",
    "Upton",
    "Uma",
    "Urban",
    "Unique",
    "Udo",
    "Usher",
    "Unity",
    // V
    "Victoria",
    "Valerie",
    "Vanessa",
    "Vincent",
    "Victor",
    "Violet",
    "Vernon",
    "Vivian",
    "Vaughn",
    "Valencia",
    // W
    "William",
    "Warren",
    "Wesley",
    "Wyatt",
    "Walter",
    "Winona",
    "Wayne",
    "Willow",
    "Winston",
    "Wanda",
    // X
    "Xavier",
    "Xena",
    "Ximena",
    "Xiomara",
    "Xander",
    "Xanthe",
    "Xavi",
    "Xenia",
    "Xiaowen",
    "Xyla",
    // Y
    "Yvonne",
    "Yolanda",
    "Yasmine",
    "Yancy",
    "Yara",
    "York",
    "Yusuf",
    "Yannick",
    "Yosef",
    "Yuri",
    // Z
    "Zachary",
    "Zoe",
    "Zelda",
    "Zara",
    "Zeke",
    "Zena",
    "Ziggy",
    "Zola",
    "Zuri",
    "Zane",
];

pub struct FirstNameSubstitutor {}
impl FunctionSubstitutor for FirstNameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bfirst_name\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..FIRST_NAMES.len());
        FIRST_NAMES[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_name_substitutor() {
        let sub = FirstNameSubstitutor {};
        let first_name = sub.generate();
        assert!(
            !first_name.is_empty(),
            "Generated first name should not be empty"
        );
    }

    #[test]
    fn test_first_name_substitutor_regex() {
        let sub = FirstNameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("first_name()"));
        assert!(regex.is_match("Person: first_name()"));
        assert!(!regex.is_match("nofirst_name()"));
        assert!(!regex.is_match("myfirst_name()"));
    }

    #[test]
    fn test_first_name_word_boundary_strict() {
        let sub = FirstNameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("first_nameextra()"));
        assert!(!regex.is_match("prefixfirst_name()"));
    }

    #[test]
    fn test_first_name_generates_valid_names() {
        let sub = FirstNameSubstitutor {};
        for _ in 0..50 {
            let name = sub.generate();
            assert!(!name.is_empty(), "First name should not be empty");
            assert!(
                !name.is_empty(),
                "First name should have at least one character"
            );
        }
    }
}
