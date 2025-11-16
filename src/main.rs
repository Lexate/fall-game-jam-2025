#![expect(dead_code)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame, layout::Position, layout::Rect, widgets::Widget};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use std::io::{self, Write};

use regex::Regex;
use std::fs::File;
use std::process::{Command, Output};

use std::time::SystemTime;

mod problems;
use problems::{Language, Problem};

fn main() -> io::Result<()> {
    if !test_compilers() {
        println!("You need to have Bun and python3 installed to run this game");
    }

    // let utp = compiler::<languages::Python>("hello".to_string(), "print('helawlo!')".to_string());
    // println!("output: {}, success: {}", utp.output, utp.success);

    let prob1 = Problem {
        request: "Make the program compile. (problem stolen from rustlings)".to_string(),
        initial_problem: "struct Book {\n    author: &str,\n    title: &str,\n}\n\nfn main() {\n    let book = Book {\n        author: \"George Orwell\",\n        title: \"1984\",\n    };\n\n    println!(\"{} by {}\", book.title, book.author);\n}".to_string(),
        check_regex: "1984 by George Orwell".to_string(),
        language: Language::Rust
    };

    let prob2 = Problem {
        request: "It's just hello world".to_string(),
        initial_problem: "print('test')".to_string(),
        check_regex: "Hello world".to_string(),
        language: Language::Python,
    };

    let probs = vec![prob1, prob2].into_iter();

    let mut terminal = ratatui::init();
    let mut app = App::new(probs);

    let app_result = app.run(&mut terminal);
    ratatui::restore();
    println!("{}", app.get_editor_content());

    app_result
}

struct App<T: Iterator<Item = Problem>> {
    editor: Editor,
    editor_area: Rect,
    counter: u8,
    exit: bool,
    problems: T,
    current_prob: Problem,
    output: String,
    correct: bool,
    start_time: SystemTime,
}

impl<T: Iterator<Item = Problem>> App<T> {
    fn new(mut problems: T) -> Self {
        let init_prob = problems.next().expect("Passed in empty iterator");
        App {
            editor: Editor::new(
                &init_prob.language.name_string(),
                &init_prob.initial_problem,
                vesper(),
            ),
            editor_area: Rect::default(),
            counter: 0,
            exit: false,
            problems,
            current_prob: init_prob,
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
            KeyCode::F(1) => self.next_problem(),
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
        let result = compiler(&self.current_prob, self.get_editor_content());

        self.output = result.output;
        self.correct = result.success;
    }

    fn get_editor_content(&self) -> String {
        self.editor.get_content()
    }

    fn set_editor_content(&mut self, content: &str) {
        self.editor.set_content(content);
    }

    fn next_problem(&mut self) {
        if !self.correct {
            return;
        }
        let prob = match self.problems.next() {
            Some(p) => p,
            None => return,
        };

        self.correct = false;
        self.editor = Editor::new(
            &prob.language.name_string(),
            &prob.initial_problem,
            vesper(),
        );
        self.current_prob = prob;
    }

    fn get_status_bar(&self) -> Text<'_> {
        let cur_time = SystemTime::now();
        let status = Text::raw(format!(
            "Score: {}",
            self.current_prob.diff(self.get_editor_content()),
        ));
        status
    }
}

impl<T: Iterator<Item = Problem>> Widget for &App<T> {
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
        Paragraph::new(self.current_prob.request.clone())
            .centered()
            .block(Block::bordered().borders(Borders::BOTTOM))
            .render(areas[0], buf);

        self.editor.render(areas[1], buf);

        Paragraph::new(self.output.clone())
            .block(Block::bordered().title("Output"))
            .render(areas[2], buf);

        Paragraph::new(self.get_status_bar()).render(areas[3], buf);

        if self.correct {
            let block = Paragraph::new("weeee\nslask")
                .centered()
                .block(Block::bordered().title("popup"));
            let popup_area = popup_area(area, 60, 20);
            block.render(popup_area, buf);
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
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

fn compiler(problem: &Problem, input: String) -> CompilerReturn {
    let name = "temp.tmp";
    let mut file = File::create(name).unwrap();
    write!(file, "{}", input).unwrap();

    let utp = Command::new("sh")
        .arg("-c")
        .arg(problem.language.format_command(name))
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
        .arg(format!("rm ./{name} {}", problem.language.clean_up()))
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
