#![expect(dead_code)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, layout::Position, layout::Rect, widgets::Widget};
use ratatui_code_editor::editor::Editor;
use ratatui_code_editor::theme::vesper;
use std::io;
use std::rc::Rc;

use std::process::Command;

fn main() -> anyhow::Result<()> {
    let mut spec_command = if cfg!(target_os = "windows") {
        Command::new("cmd")
    } else {
        Command::new("sh")
    };
    if !test_compilers(&mut spec_command) {
        println!("You need to have Bun and python3 installed to run this game");
    }

    let mut terminal = ratatui::init();
    let mut app = App::new();
    // let app_result = app.run(&mut terminal);
    ratatui::restore();
    // app_result
    println!("{}", app.get_editor_content());
    Ok(())
}

#[derive()]
struct App {
    editor: Editor,
    editor_area: Rect,
    counter: u8,
    exit: bool,
}

impl App {
    fn new() -> Self {
        App {
            editor: Editor::new("rust", "test", vesper()),
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
            frame.set_cursor_position(Position::new(x, y));
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
            _ => self.editor.input(key_event, &self.editor_area).unwrap(),
        }
        Ok(())
    }

    fn set_area(&mut self, area: Rect) {
        self.editor_area = area;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_editor_content(&self) -> String {
        self.editor.get_content()
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.editor.render(area, buf);
    }
}

fn test_compilers(command: &mut Command) -> bool {
    let output = command
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
