mod tui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Padding, Paragraph, Widget,
    },
    Frame,
};

use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
};

use rand::seq::SliceRandom;

#[derive(Debug, Default)]
pub struct App {
    target_word: String,
    // 0,1,2,3, or 4 (basically the rows (just add 1 to index))
    // Default: 0
    round: u8,
    reveal_answer: bool,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        self.choose_random_word()
            .expect("Could not get random word!");

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn choose_random_word(&mut self) -> Result<()> {
        let path = "fiveletterwords.txt";
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let words: Vec<String> = reader.lines().filter_map(Result::ok).collect();

        if words.is_empty() {
            eprintln!("The file is empty or cannot be read.");
        }

        let mut rng = rand::thread_rng();
        if let Some(random_word) = words.choose(&mut rng) {
            self.target_word = random_word.clone();
            Ok(())
        } else {
            eprintln!("Could not get random word!");
            Ok(())
        }
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self) -> Result<()> {
        let _ = match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => Ok(()),
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.exit()
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.choose_random_word()
                    .expect("Could not reload random word!");
            }
            KeyCode::Char('v') | KeyCode::Char('V')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.reveal_answer = !self.reveal_answer
            }
            _ => (),
        };

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Worlde TUI in Rust ".bold());
        let instruction = Title::from(Line::from(vec![
            " <Ctrl-r> to refresh word ".blue(),
            " <Ctrl-v> to reveal word ".blue(),
            " <Ctrl-q> to exit ".blue(),
        ]));

        let top_block = Block::bordered()
            .border_style(Style::default().blue())
            .title(title.alignment(Alignment::Center))
            .title(
                instruction
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .padding(Padding::vertical(2))
            .border_set(border::THICK);

        let current_word_text = Text::from(vec![Line::from(vec![if self.reveal_answer {
            self.target_word.as_str().red().underlined()
        } else {
            "".into()
        }])]);

        Paragraph::new(current_word_text)
            .centered()
            .block(top_block)
            .render(area, buf);
    }
}

fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    // App::default() will init app with counter set to 0, and exit set to false
    let _ = App::default().run(&mut terminal);
    let _ = tui::restore();

    Ok(())
}
