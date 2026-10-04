use crate::functions::substitution::FunctionSubstitutor;

/// Country name, ISO 3166-1 alpha-2 code, ISO 3166-1 alpha-3 code.
/// Entries are in the same order as `country::COUNTRIES`.
pub(crate) static COUNTRY_CODES: &[(&str, &str, &str)] = &[
    ("Afghanistan", "AF", "AFG"),
    ("Albania", "AL", "ALB"),
    ("Algeria", "DZ", "DZA"),
    ("Andorra", "AD", "AND"),
    ("Angola", "AO", "AGO"),
    ("Antigua and Barbuda", "AG", "ATG"),
    ("Argentina", "AR", "ARG"),
    ("Armenia", "AM", "ARM"),
    ("Australia", "AU", "AUS"),
    ("Austria", "AT", "AUT"),
    ("Azerbaijan", "AZ", "AZE"),
    ("Bahamas", "BS", "BHS"),
    ("Bahrain", "BH", "BHR"),
    ("Bangladesh", "BD", "BGD"),
    ("Barbados", "BB", "BRB"),
    ("Belarus", "BY", "BLR"),
    ("Belgium", "BE", "BEL"),
    ("Belize", "BZ", "BLZ"),
    ("Benin", "BJ", "BEN"),
    ("Bhutan", "BT", "BTN"),
    ("Bolivia", "BO", "BOL"),
    ("Bosnia and Herzegovina", "BA", "BIH"),
    ("Botswana", "BW", "BWA"),
    ("Brazil", "BR", "BRA"),
    ("Brunei", "BN", "BRN"),
    ("Bulgaria", "BG", "BGR"),
    ("Burkina Faso", "BF", "BFA"),
    ("Burundi", "BI", "BDI"),
    ("Cabo Verde", "CV", "CPV"),
    ("Cambodia", "KH", "KHM"),
    ("Cameroon", "CM", "CMR"),
    ("Canada", "CA", "CAN"),
    ("Central African Republic", "CF", "CAF"),
    ("Chad", "TD", "TCD"),
    ("Chile", "CL", "CHL"),
    ("China", "CN", "CHN"),
    ("Colombia", "CO", "COL"),
    ("Comoros", "KM", "COM"),
    ("Congo", "CG", "COG"),
    ("Costa Rica", "CR", "CRI"),
    ("Croatia", "HR", "HRV"),
    ("Cuba", "CU", "CUB"),
    ("Cyprus", "CY", "CYP"),
    ("Czech Republic", "CZ", "CZE"),
    ("Democratic Republic of the Congo", "CD", "COD"),
    ("Denmark", "DK", "DNK"),
    ("Djibouti", "DJ", "DJI"),
    ("Dominica", "DM", "DMA"),
    ("Dominican Republic", "DO", "DOM"),
    ("Ecuador", "EC", "ECU"),
    ("Egypt", "EG", "EGY"),
    ("El Salvador", "SV", "SLV"),
    ("Equatorial Guinea", "GQ", "GNQ"),
    ("Eritrea", "ER", "ERI"),
    ("Estonia", "EE", "EST"),
    ("Eswatini", "SZ", "SWZ"),
    ("Ethiopia", "ET", "ETH"),
    ("Fiji", "FJ", "FJI"),
    ("Finland", "FI", "FIN"),
    ("France", "FR", "FRA"),
    ("Gabon", "GA", "GAB"),
    ("Gambia", "GM", "GMB"),
    ("Georgia", "GE", "GEO"),
    ("Germany", "DE", "DEU"),
    ("Ghana", "GH", "GHA"),
    ("Greece", "GR", "GRC"),
    ("Grenada", "GD", "GRD"),
    ("Guatemala", "GT", "GTM"),
    ("Guinea", "GN", "GIN"),
    ("Guinea-Bissau", "GW", "GNB"),
    ("Guyana", "GY", "GUY"),
    ("Haiti", "HT", "HTI"),
    ("Honduras", "HN", "HND"),
    ("Hungary", "HU", "HUN"),
    ("Iceland", "IS", "ISL"),
    ("India", "IN", "IND"),
    ("Indonesia", "ID", "IDN"),
    ("Iran", "IR", "IRN"),
    ("Iraq", "IQ", "IRQ"),
    ("Ireland", "IE", "IRL"),
    ("Israel", "IL", "ISR"),
    ("Italy", "IT", "ITA"),
    ("Ivory Coast", "CI", "CIV"),
    ("Jamaica", "JM", "JAM"),
    ("Japan", "JP", "JPN"),
    ("Jordan", "JO", "JOR"),
    ("Kazakhstan", "KZ", "KAZ"),
    ("Kenya", "KE", "KEN"),
    ("Kiribati", "KI", "KIR"),
    ("Kuwait", "KW", "KWT"),
    ("Kyrgyzstan", "KG", "KGZ"),
    ("Laos", "LA", "LAO"),
    ("Latvia", "LV", "LVA"),
    ("Lebanon", "LB", "LBN"),
    ("Lesotho", "LS", "LSO"),
    ("Liberia", "LR", "LBR"),
    ("Libya", "LY", "LBY"),
    ("Liechtenstein", "LI", "LIE"),
    ("Lithuania", "LT", "LTU"),
    ("Luxembourg", "LU", "LUX"),
    ("Madagascar", "MG", "MDG"),
    ("Malawi", "MW", "MWI"),
    ("Malaysia", "MY", "MYS"),
    ("Maldives", "MV", "MDV"),
    ("Mali", "ML", "MLI"),
    ("Malta", "MT", "MLT"),
    ("Marshall Islands", "MH", "MHL"),
    ("Mauritania", "MR", "MRT"),
    ("Mauritius", "MU", "MUS"),
    ("Mexico", "MX", "MEX"),
    ("Micronesia", "FM", "FSM"),
    ("Moldova", "MD", "MDA"),
    ("Monaco", "MC", "MCO"),
    ("Mongolia", "MN", "MNG"),
    ("Montenegro", "ME", "MNE"),
    ("Morocco", "MA", "MAR"),
    ("Mozambique", "MZ", "MOZ"),
    ("Myanmar", "MM", "MMR"),
    ("Namibia", "NA", "NAM"),
    ("Nauru", "NR", "NRU"),
    ("Nepal", "NP", "NPL"),
    ("Netherlands", "NL", "NLD"),
    ("New Zealand", "NZ", "NZL"),
    ("Nicaragua", "NI", "NIC"),
    ("Niger", "NE", "NER"),
    ("Nigeria", "NG", "NGA"),
    ("North Korea", "KP", "PRK"),
    ("North Macedonia", "MK", "MKD"),
    ("Norway", "NO", "NOR"),
    ("Oman", "OM", "OMN"),
    ("Pakistan", "PK", "PAK"),
    ("Palau", "PW", "PLW"),
    ("Palestine", "PS", "PSE"),
    ("Panama", "PA", "PAN"),
    ("Papua New Guinea", "PG", "PNG"),
    ("Paraguay", "PY", "PRY"),
    ("Peru", "PE", "PER"),
    ("Philippines", "PH", "PHL"),
    ("Poland", "PL", "POL"),
    ("Portugal", "PT", "PRT"),
    ("Qatar", "QA", "QAT"),
    ("Romania", "RO", "ROU"),
    ("Russia", "RU", "RUS"),
    ("Rwanda", "RW", "RWA"),
    ("Saint Kitts and Nevis", "KN", "KNA"),
    ("Saint Lucia", "LC", "LCA"),
    ("Saint Vincent and the Grenadines", "VC", "VCT"),
    ("Samoa", "WS", "WSM"),
    ("San Marino", "SM", "SMR"),
    ("Sao Tome and Principe", "ST", "STP"),
    ("Saudi Arabia", "SA", "SAU"),
    ("Senegal", "SN", "SEN"),
    ("Serbia", "RS", "SRB"),
    ("Seychelles", "SC", "SYC"),
    ("Sierra Leone", "SL", "SLE"),
    ("Singapore", "SG", "SGP"),
    ("Slovakia", "SK", "SVK"),
    ("Slovenia", "SI", "SVN"),
    ("Solomon Islands", "SB", "SLB"),
    ("Somalia", "SO", "SOM"),
    ("South Africa", "ZA", "ZAF"),
    ("South Korea", "KR", "KOR"),
    ("South Sudan", "SS", "SSD"),
    ("Spain", "ES", "ESP"),
    ("Sri Lanka", "LK", "LKA"),
    ("Sudan", "SD", "SDN"),
    ("Suriname", "SR", "SUR"),
    ("Sweden", "SE", "SWE"),
    ("Switzerland", "CH", "CHE"),
    ("Syria", "SY", "SYR"),
    ("Tajikistan", "TJ", "TJK"),
    ("Tanzania", "TZ", "TZA"),
    ("Thailand", "TH", "THA"),
    ("Timor-Leste", "TL", "TLS"),
    ("Togo", "TG", "TGO"),
    ("Tonga", "TO", "TON"),
    ("Trinidad and Tobago", "TT", "TTO"),
    ("Tunisia", "TN", "TUN"),
    ("Turkey", "TR", "TUR"),
    ("Turkmenistan", "TM", "TKM"),
    ("Tuvalu", "TV", "TUV"),
    ("Uganda", "UG", "UGA"),
    ("Ukraine", "UA", "UKR"),
    ("United Arab Emirates", "AE", "ARE"),
    ("United Kingdom", "GB", "GBR"),
    ("United States", "US", "USA"),
    ("Uruguay", "UY", "URY"),
    ("Uzbekistan", "UZ", "UZB"),
    ("Vanuatu", "VU", "VUT"),
    ("Vatican City", "VA", "VAT"),
    ("Venezuela", "VE", "VEN"),
    ("Vietnam", "VN", "VNM"),
    ("Yemen", "YE", "YEM"),
    ("Zambia", "ZM", "ZMB"),
    ("Zimbabwe", "ZW", "ZWE"),
];

pub struct CountrySubstitutor {}

impl FunctionSubstitutor for CountrySubstitutor {
    fn get_regex(&self) -> &str {
        r"\bcountry\(\)"
    }

    fn generate(&self) -> String {
        random_country().0.to_string()
    }
}

pub struct CountryAlpha2Substitutor {}

impl FunctionSubstitutor for CountryAlpha2Substitutor {
    fn get_regex(&self) -> &str {
        r"\bcountrycode2\(\)"
    }

    fn generate(&self) -> String {
        random_country().1.to_string()
    }
}

pub struct CountryAlpha3Substitutor {}

impl FunctionSubstitutor for CountryAlpha3Substitutor {
    fn get_regex(&self) -> &str {
        r"\bcountrycode3\(\)"
    }

    fn generate(&self) -> String {
        random_country().2.to_string()
    }
}

fn random_country() -> &'static (&'static str, &'static str, &'static str) {
    use rand::RngExt;
    let mut rng = rand::rng();
    &COUNTRY_CODES[rng.random_range(0..COUNTRY_CODES.len())]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_country_substitutor_generates_known_country() {
        let country = CountrySubstitutor {}.generate();
        assert!(COUNTRY_CODES.iter().any(|c| c.0 == country));
    }

    #[test]
    fn test_countries_are_complete_and_unique() {
        assert_eq!(COUNTRY_CODES.len(), 195);
        let names: HashSet<_> = COUNTRY_CODES.iter().map(|c| c.0).collect();
        assert_eq!(names.len(), COUNTRY_CODES.len(), "duplicate country names");
        assert!(
            COUNTRY_CODES
                .iter()
                .all(|c| !c.0.trim().is_empty() && c.0.trim() == c.0)
        );
    }

    #[test]
    fn test_codes_are_well_formed_and_unique() {
        let upper =
            |s: &str, len: usize| s.len() == len && s.chars().all(|c| c.is_ascii_uppercase());
        for (name, a2, a3) in COUNTRY_CODES {
            assert!(upper(a2, 2), "bad alpha-2 '{}' for {}", a2, name);
            assert!(upper(a3, 3), "bad alpha-3 '{}' for {}", a3, name);
        }
        let a2: HashSet<_> = COUNTRY_CODES.iter().map(|c| c.1).collect();
        let a3: HashSet<_> = COUNTRY_CODES.iter().map(|c| c.2).collect();
        assert_eq!(a2.len(), COUNTRY_CODES.len(), "duplicate alpha-2 codes");
        assert_eq!(a3.len(), COUNTRY_CODES.len(), "duplicate alpha-3 codes");
    }

    #[test]
    fn test_known_codes() {
        let find = |n: &str| COUNTRY_CODES.iter().find(|c| c.0 == n).unwrap();
        assert_eq!(find("Denmark"), &("Denmark", "DK", "DNK"));
        assert_eq!(find("United States"), &("United States", "US", "USA"));
        assert_eq!(find("United Kingdom"), &("United Kingdom", "GB", "GBR"));
        assert_eq!(find("Germany"), &("Germany", "DE", "DEU"));
    }

    #[test]
    fn test_alpha2_substitutor() {
        let value = CountryAlpha2Substitutor {}.generate();
        assert!(COUNTRY_CODES.iter().any(|c| c.1 == value));
    }

    #[test]
    fn test_alpha3_substitutor() {
        let value = CountryAlpha3Substitutor {}.generate();
        assert!(COUNTRY_CODES.iter().any(|c| c.2 == value));
    }

    #[test]
    fn test_regexes_match_only_exact_function_calls() {
        let country = regex::Regex::new(CountrySubstitutor {}.get_regex()).unwrap();
        let a2 = regex::Regex::new(CountryAlpha2Substitutor {}.get_regex()).unwrap();
        let a3 = regex::Regex::new(CountryAlpha3Substitutor {}.get_regex()).unwrap();
        assert!(country.is_match("country()") && !country.is_match("mycountry()"));
        assert!(!country.is_match("countrycode2()"));
        assert!(a2.is_match("countrycode2()") && !a2.is_match("countrycode3()"));
        assert!(a3.is_match("countrycode3()") && !a3.is_match("countrycode2()"));
    }
}
