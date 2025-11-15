#![expect(dead_code)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame, layout::Position, layout::Rect, widgets::Widget};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use std::io;
use std::io::{self, Write};
use std::rc::Rc;

use regex::Regex;
use std::fs::{self, File};
use std::process::{Command, Output};
fn main() -> anyhow::Result<()> {
    if !test_compilers() {
        println!("You need to have Bun and python3 installed to run this game");
    }

    let utp = compiler(
        "hello".to_string(),
        "print('helawlo!')".to_string(),
        "python3",
    );
    println!("output: {}, success: {}", utp.output, utp.success);

    let mut terminal = ratatui::init();
    let mut app = App::new();

    // let app_result = app.run(&mut terminal);
    ratatui::restore();
    // app_result
    println!("{}", app.get_editor_content());
    Ok(())
}

struct App {
    editor: Editor,
    editor_area: Rect,
    counter: u8,
    exit: bool,
}

impl App {
    fn new() -> Self {
        App {
            editor: Editor::new(
                "rust",
                "fn main() {\n    println!(\"Hello, world!\");\n}",
                vesper(),
            ),
            editor_area: Rect::default(),
            counter: 0,
            exit: false,
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

    fn check(&self) {
        todo!()
    }

    fn get_editor_content(&self) -> String {
        self.editor.get_content()
    }

    fn set_editor_content(&mut self, content: &str) {
        self.editor.set_content(content);
    }
}

impl Widget for &App {
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
        Paragraph::new("problem")
            .centered()
            .block(Block::bordered().borders(Borders::BOTTOM))
            .render(areas[0], buf);

        self.editor.render(areas[1], buf);

        Paragraph::new(self.get_editor_content())
            .block(Block::bordered().title("Output"))
            .render(areas[2], buf);

        Paragraph::new("Stats").render(areas[3], buf);
    }
}

struct Problem {
    initial_problem: String,
    check_regex: String,
}

impl Problem {
    fn diff(&self, comparison: String) -> usize {
        todo!()
    }
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
fn compiler(regex: String, input: String, language: &str) -> CompilerReturn {
    let name = "temp.tmp";
    let mut file = File::create(name).unwrap();
    write!(file, "{}", input).unwrap();

    let utp = Command::new("sh")
        .arg("-c")
        .arg(format!("{language} {name}"))
        .output()
        .unwrap();

    let error_or_message = String::from_utf8(pretty_print_output(utp)).unwrap();

    let re = Regex::new(&regex).unwrap();

    let cap = match re.captures(&error_or_message) {
        Some(t) => t.len() != 0,
        None => false,
    };
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
