use std::fs;
use std::{mem::swap, str::FromStr, vec};
pub struct Problem {
    pub request: String,
    pub initial_problem: String,
    pub language: Language,
    pub check_regex: String,
}

impl Problem {
    pub fn diff(&self, comparison: String) -> usize {
        minimum_edit_distance(&self.initial_problem, &comparison)
    }

    pub fn get_regex(&self) -> &str {
        &self.check_regex
    }
}

pub enum Language {
    Rust,
    Python,
    TypeScript,
}

impl Language {
    pub fn format_command(&self, file_name: &str) -> String {
        match self {
            Self::Rust => format!("rustc {file_name} -o temp_prog && ./temp_prog"),
            Self::Python => format!("python3 {file_name}"),
            Self::TypeScript => format!("bun run {file_name}"),
        }
    }

    pub fn name_string(&self) -> String {
        match self {
            Self::Rust => "rust".to_string(),
            Self::Python => "python".to_string(),
            Self::TypeScript => "typescript".to_string(),
        }
    }

    pub fn clean_up(&self) -> String {
        match self {
            Self::Rust => "./temp_prog".to_string(),
            _ => "".to_string(),
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

pub fn create_problems() -> Vec<Problem> {
    let inp = match fs::read_to_string("./uppg/NR.py") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{e}");
            "".to_string()
        }
    };
    let inp2 = match fs::read_to_string("./uppg/preb.ts") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{e}");
            "".to_string()
        }
    };
    let prob1: Problem = Problem {
        request: "Make sure that the algorithm does not converge in 25 iterations".to_string(),
        initial_problem: inp,
        language: Language::Python,
        check_regex: r"Did not converge :/".to_string(),
    };
    let prob2: Problem = Problem {
        request: "Fix the code so that the async generator finishes".to_string(),
        initial_problem: inp2,
        language: Language::TypeScript,
        check_regex: r"done!$".to_string(),
    };
    let prob3= Problem {
        request: "Make the program compile. (problem stolen from rustlings)".to_string(),
        initial_problem: "struct Book {\n    author: &str,\n    title: &str,\n}\n\nfn main() {\n    let book = Book {\n        author: \"George Orwell\",\n        title: \"1984\",\n    };\n\n    println!(\"{} by {}\", book.title, book.author);\n}".to_string(),
        check_regex: "1984 by George Orwell".to_string(),
        language: Language::Rust
    };
    vec![prob1, prob2, prob3]
}
