use std::mem::swap;

pub struct Problem<L>
where
    L: languages::Language,
{
    pub request: String,
    pub initial_problem: String,
    pub language: L,
    pub check_regex: String,
}

impl<L> Problem<L>
where
    L: languages::Language,
{
    pub fn diff(&self, comparison: String) -> usize {
        minimum_edit_distance(&self.initial_problem, &comparison)
    }

    pub fn get_regex(&self) -> &str {
        &self.check_regex
    }
}

pub mod languages {
    pub trait Language {
        fn format_command(file_name: &str) -> String;

        fn name_string() -> String;

        fn clean_up() -> String {
            "".to_string()
        }
    }
    pub struct Python;

    impl Language for Python {
        fn format_command(file_name: &str) -> String {
            format!("python3 {file_name}")
        }

        fn name_string() -> String {
            "python".to_string()
        }
    }

    pub struct Rust;

    impl Language for Rust {
        fn format_command(file_name: &str) -> String {
            format!("rustc {file_name} -o temp_prog && ./temp_prog")
        }

        fn name_string() -> String {
            "rust".to_string()
        }

        fn clean_up() -> String {
            "./temp_prog".to_string()
        }
    }

    pub struct Typescript;
    impl Language for Typescript {
        fn format_command(file_name: &str) -> String {
            format!("bun run {file_name}")
        }

        fn name_string() -> String {
            "typescript".to_string()
        }
    }
}

fn minimum_edit_distance(s1: &str, s2: &str) -> usize {
    let s: Vec<char> = s1.chars().collect();
    let t: Vec<char> = s2.chars().collect();
    let s_len = s.len();
    let t_len = t.len();

    let mut v0: Vec<usize> = (0..t_len + 1).collect();
    let mut v1: Vec<usize> = vec![0; t_len + 1];

    let mut substitution_cost: usize;

    for i in 0..s_len {
        v1[0] = i + 1;

        for j in 0..t_len {
            let deletion_cost = v0[j + 1] + 1;
            let insertion_cost = v1[j] + 1;

            if s[i] == t[j] {
                substitution_cost = v0[j];
            } else {
                substitution_cost = v0[j] + 1;
            }

            v1[j + 1] = deletion_cost.min(insertion_cost).min(substitution_cost)
        }

        swap(&mut v0, &mut v1);
    }
    v0[t_len]
}
