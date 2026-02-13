use super::substitution::FunctionSubstitutor;

pub(crate) static JOB_TITLES: &[&str] = &[
    "Software Engineer",
    "Data Scientist",
    "Product Manager",
    "Graphic Designer",
    "Marketing Specialist",
    "Sales Representative",
    "Human Resources Manager",
    "Financial Analyst",
    "Customer Service Representative",
    "Operations Manager",
    "Business Analyst",
    "Content Writer",
    "UX/UI Designer",
    "Project Coordinator",
    "Quality Assurance Tester",
    "Social Media Manager",
    "IT Support Specialist",
    "Accountant",
    "Web Developer",
    "Digital Marketing Manager",
];

pub struct JobTitleSubstitutor {}
impl FunctionSubstitutor for JobTitleSubstitutor {
    fn get_regex(&self) -> &str {
        r"\bjob_title\(\)"
    }

    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..JOB_TITLES.len());
        JOB_TITLES[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_title_substitutor() {
        let sub = JobTitleSubstitutor {};
        let job_title = sub.generate();
        assert!(
            !job_title.is_empty(),
            "Generated job title should not be empty"
        );
    }

    #[test]
    fn test_job_title_substitutor_generates_different_values() {
        let sub = JobTitleSubstitutor {};
        let job1 = sub.generate();
        let job2 = sub.generate();

        assert!(!job1.is_empty());
        assert!(!job2.is_empty());
    }
}
