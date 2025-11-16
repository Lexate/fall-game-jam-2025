#![expect(dead_code)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame, layout::Position, layout::Rect, widgets::Widget};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use std::io::{self, Write};
use std::mem::swap;

use regex::Regex;
use std::fs::File;
use std::process::{Command, Output};

use std::time::SystemTime;

fn main() -> io::Result<()> {
    if !test_compilers() {
        println!("You need to have Bun and python3 installed to run this game");
    }

    // let utp = compiler::<languages::Python>("hello".to_string(), "print('helawlo!')".to_string());
    // println!("output: {}, success: {}", utp.output, utp.success);

    let prob = Problem {
        request: "Make the program compile. (problem stolen from rustlings)".to_string(),
        initial_problem: "struct Book {\n    author: &str,\n    title: &str,\n}\n\nfn main() {\n    let book = Book {\n        author: \"George Orwell\",\n        title: \"1984\",\n    };\n\n    println!(\"{} by {}\", book.title, book.author);\n}".to_string(),
        check_regex: "1984 by George Orwell".to_string(),
        language: languages::Rust
    };

    let mut terminal = ratatui::init();
    let mut app = App::new(prob);

    let app_result = app.run(&mut terminal);
    ratatui::restore();
    println!("{}", app.get_editor_content());

    app_result
}

struct App<L>
where
    L: languages::Language,
{
    editor: Editor,
    editor_area: Rect,
    counter: u8,
    exit: bool,
    problem: Problem<L>,
    output: String,
    correct: bool,
    start_time: SystemTime,
}

impl<L> App<L>
where
    L: languages::Language,
{
    fn new(problem: Problem<L>) -> Self {
        App {
            editor: Editor::new("rust", &problem.initial_problem, vesper()),
            editor_area: Rect::default(),
            counter: 0,
            exit: false,
            problem,
            output: "Press f5 to check your code against the solution.".to_string(),
            correct: false,
            start_time: SystemTime::now(),
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(&*self, area);

        let cursor = self.editor.get_visible_cursor(&area);
        if let Some((x, y)) = cursor {
            frame.set_cursor_position(Position::new(x, y + 3));
        }
        self.set_area(area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)?
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::F(5) => self.check(),
            _ => self.editor.input(key_event, &self.editor_area).unwrap(),
        }
        Ok(())
    }

    fn set_area(&mut self, area: Rect) {
        let areas = Layout::vertical([
            Constraint::Max(3),
            Constraint::Fill(1),
            Constraint::Max(20),
            Constraint::Max(1),
        ])
        .split(area);
        self.editor_area = areas[1];
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn check(&mut self) {
        let result = compiler::<L>(&self.problem, self.get_editor_content());

        self.output = result.output;
        self.correct = result.success;
    }

    fn get_editor_content(&self) -> String {
        self.editor.get_content()
    }

    fn set_editor_content(&mut self, content: &str) {
        self.editor.set_content(content);
    }

    fn get_status_bar(&self) -> Text<'_> {
        let cur_time = SystemTime::now();
        let status = Text::raw(format!(
            "Score: {}, Sucess: {}, Time Elapsed: {}",
            self.problem.diff(self.get_editor_content()),
            self.correct,
            cur_time.duration_since(self.start_time).unwrap().as_secs()
        ));
        status
    }
}

impl<L> Widget for &App<L>
where
    L: languages::Language,
{
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let areas = Layout::vertical([
            Constraint::Max(3),
            Constraint::Fill(1),
            Constraint::Max(20),
            Constraint::Max(1),
        ])
        .split(area);
        Paragraph::new(self.problem.request.clone())
            .centered()
            .block(Block::bordered().borders(Borders::BOTTOM))
            .render(areas[0], buf);

        self.editor.render(areas[1], buf);

        Paragraph::new(self.output.clone())
            .block(Block::bordered().title("Output"))
            .render(areas[2], buf);

        Paragraph::new(self.get_status_bar()).render(areas[3], buf);
    }
}

struct Problem<L>
where
    L: languages::Language,
{
    request: String,
    initial_problem: String,
    language: L,
    check_regex: String,
}

impl<L> Problem<L>
where
    L: languages::Language,
{
    fn diff(&self, comparison: String) -> usize {
        minimum_edit_distance(&self.initial_problem, &comparison)
    }

    fn get_regex(&self) -> &str {
        &self.check_regex
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

fn test_compilers() -> bool {
    let output = Command::new("sh")
        .args([
            "-c",
            "python3",
            "--version",
            "&&",
            "bun",
            "--version",
            "&&",
            "rustc",
            "--version",
        ])
        .output()
        .expect("Could not execute tests");
    let cp = output.clone();
    match String::from_utf8(cp.stderr) {
        Ok(data) => println!("Err: {}", data),
        Err(e) => eprintln!("{}", e),
    }
    return output.stderr.len() < 1;
}

struct CompilerReturn {
    success: bool,
    output: String,
}

mod languages {
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
}

fn compiler<L>(problem: &Problem<L>, input: String) -> CompilerReturn
where
    L: languages::Language,
{
    let name = "temp.tmp";
    let mut file = File::create(name).unwrap();
    write!(file, "{}", input).unwrap();

    let utp = Command::new("sh")
        .arg("-c")
        .arg(L::format_command(name))
        .output()
        .unwrap();

    let error_or_message = String::from_utf8(pretty_print_output(utp)).unwrap();

    let re = Regex::new(&problem.get_regex()).unwrap();

    let cap = match re.captures(&error_or_message) {
        Some(t) => t.len() != 0,
        None => false,
    };

    Command::new("sh")
        .arg("-c")
        .arg(format!("rm ./{name} {}", L::clean_up()))
        .output()
        .unwrap();
    // println!("Test: {}", error_or_message);
    return CompilerReturn {
        success: cap,
        output: error_or_message,
    };
}

fn pretty_print_output(output: Output) -> Vec<u8> {
    if output.stderr.len() > 1 {
        return output.stderr;
    } else {
        return output.stdout;
    }
}
