use super::substitution::FunctionSubstitutor;

pub(crate) static LAST_NAMES: &[&str] = &[
    // A
    "Abbott",
    "Adams",
    "Adkins",
    "Aguilar",
    "Aguirre",
    "Alexander",
    "Allison",
    "Almanza",
    "Anderson",
    "Andrews",
    // B
    "Bailey",
    "Baker",
    "Baldwin",
    "Ballard",
    "Barnard",
    "Barnes",
    "Barrett",
    "Barron",
    "Barton",
    "Bates",
    // C
    "Cain",
    "Caldwell",
    "Calhoun",
    "Callahan",
    "Cameron",
    "Campbell",
    "Campfield",
    "Cannon",
    "Cantrell",
    "Carey",
    // D
    "Dalton",
    "Daniel",
    "Daniels",
    "Darby",
    "Darden",
    "Davenport",
    "Davidson",
    "Davis",
    "Dawson",
    "Dayton",
    // E
    "Eaton",
    "Eberly",
    "Eckert",
    "Edison",
    "Edmonds",
    "Edwards",
    "Efron",
    "Egan",
    "Eggleston",
    "Eisenhower",
    // F
    "Fabiano",
    "Fairbanks",
    "Fairchild",
    "Falcone",
    "Farmer",
    "Farrell",
    "Farrow",
    "Faulkner",
    "Fawcett",
    "Fay",
    // G
    "Gabel",
    "Gadson",
    "Gage",
    "Gaines",
    "Gallagher",
    "Gallery",
    "Galley",
    "Galloway",
    "Garcia",
    "Gardner",
    // H
    "Habib",
    "Hackett",
    "Hadden",
    "Hadley",
    "Hageman",
    "Hahn",
    "Haley",
    "Hall",
    "Hallmark",
    "Hamm",
    // I
    "Iannaccone",
    "Ingalls",
    "Ingram",
    "Innis",
    "Inouye",
    "Ireland",
    "Irons",
    "Irving",
    "Irwin",
    "Isaac",
    // J
    "Jackson",
    "Jacobs",
    "Jacobson",
    "Jagielski",
    "Jahner",
    "James",
    "Jameson",
    "Jarrett",
    "Jasinski",
    "Jaynes",
    // K
    "Kable",
    "Kaczmarski",
    "Kadel",
    "Kael",
    "Kagan",
    "Kahn",
    "Kale",
    "Kallenborn",
    "Kalogeris",
    "Kaminsky",
    // L
    "Lace",
    "Lacy",
    "Lade",
    "Lagrange",
    "Laine",
    "Laird",
    "Lalonde",
    "Lamb",
    "Lambert",
    "Lamm",
    // M
    "Mace",
    "Macey",
    "Macias",
    "Mack",
    "Mackenzie",
    "Maclaughlin",
    "Macon",
    "Madden",
    "Maddox",
    "Madigan",
    // N
    "Naber",
    "Nachman",
    "Nagle",
    "Nagel",
    "Nagle",
    "Nagy",
    "Nahu",
    "Naish",
    "Naitove",
    "Nakata",
    // O
    "Oakley",
    "Oakes",
    "Oaks",
    "Oatley",
    "Ober",
    "Oberon",
    "Obringer",
    "Obrzut",
    "O'Brien",
    "O'Byrne",
    // P
    "Pace",
    "Pacheco",
    "Pachter",
    "Packer",
    "Paction",
    "Padron",
    "Pagan",
    "Pager",
    "Paige",
    "Paine",
    // Q
    "Qian",
    "Qin",
    "Qing",
    "Quackenbush",
    "Quade",
    "Quail",
    "Quayle",
    "Queen",
    "Quentin",
    "Quetone",
    // R
    "Race",
    "Racette",
    "Rackley",
    "Racz",
    "Radcliff",
    "Radcliffe",
    "Radde",
    "Radford",
    "Radkey",
    "Radosevich",
    // S
    "Sable",
    "Sackett",
    "Sackler",
    "Sadler",
    "Sadowski",
    "Safford",
    "Safransky",
    "Sage",
    "Sager",
    "Sailer",
    // T
    "Tableman",
    "Tackett",
    "Tadlock",
    "Taft",
    "Taggart",
    "Tahir",
    "Tailor",
    "Taintor",
    "Taka",
    "Takacs",
    // U
    "Uecker",
    "Ueberroth",
    "Uelmen",
    "Ueno",
    "Ugarsky",
    "Uher",
    "Uhlmann",
    "Ukoha",
    "Ulbrich",
    "Ulery",
    // V
    "Vacarro",
    "Vaccaro",
    "Vache",
    "Vachon",
    "Vada",
    "Vadeboncoeur",
    "Vadi",
    "Vadivieso",
    "Vadose",
    "Vaerst",
    // W
    "Wacker",
    "Waddle",
    "Waddy",
    "Wade",
    "Wadha",
    "Wadhams",
    "Wadi",
    "Wadleigh",
    "Wadley",
    "Wadlington",
    // X
    "Xanares",
    "Xander",
    "Xandu",
    "Xanelli",
    "Xandy",
    "Xanth",
    "Xanthopoulos",
    "Xanthy",
    "Xantus",
    "Xantus-Kornfeld",
    // Y
    "Yackel",
    "Yackey",
    "Yackley",
    "Yaconiello",
    "Yacono",
    "Yager",
    "Yaggy",
    "Yahl",
    "Yaklin",
    "Yakstub",
    // Z
    "Zabel",
    "Zabka",
    "Zablocki",
    "Zaby",
    "Zachariah",
    "Zacharias",
    "Zacharin",
    "Zachary",
    "Zaccagnino",
    "Zaccarelli",
];

pub struct LastNameSubstitutor {}
impl FunctionSubstitutor for LastNameSubstitutor {
    fn get_regex(&self) -> &str {
        r"\blast_name\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..LAST_NAMES.len());
        LAST_NAMES[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_name_substitutor() {
        let sub = LastNameSubstitutor {};
        let last_name = sub.generate();
        assert!(
            !last_name.is_empty(),
            "Generated last name should not be empty"
        );
    }

    #[test]
    fn test_last_name_substitutor_regex() {
        let sub = LastNameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(regex.is_match("last_name()"));
        assert!(regex.is_match("Surname: last_name()"));
        assert!(!regex.is_match("nolast_name()"));
        assert!(!regex.is_match("mylast_name()"));
    }

    #[test]
    fn test_last_name_word_boundary_strict() {
        let sub = LastNameSubstitutor {};
        let regex = regex::Regex::new(sub.get_regex()).unwrap();

        assert!(!regex.is_match("last_nameextra()"));
        assert!(!regex.is_match("prefixlast_name()"));
    }

    #[test]
    fn test_last_name_generates_valid_names() {
        let sub = LastNameSubstitutor {};
        for _ in 0..50 {
            let name = sub.generate();
            assert!(!name.is_empty(), "Last name should not be empty");
            assert!(
                !name.is_empty(),
                "Last name should have at least one character"
            );
        }
    }
}
