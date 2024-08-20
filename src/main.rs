mod tui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
    },
    Frame,
};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

#[derive(Debug, Default)]
pub struct App {
    counter: i64,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("Handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('j') => self.add_to_counter(-1),
            KeyCode::Char('k') => self.add_to_counter(1),
            KeyCode::Char('J') => self.add_to_counter(-10),
            KeyCode::Char('K') => self.add_to_counter(10),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn add_to_counter(&mut self, interval: i64) {
        self.counter += interval as i64;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" Counter App Tutorial ".bold());
        let instruction = Title::from(Line::from(vec![
            " Decrement by 1 ".blue(),
            "<j>".blue().bold(),
            " Decrement by 10 ".blue(),
            "<J>".blue().bold(),
            " Increment by 10 ".blue(),
            "<K>".blue().bold(),
            " Increment by 1 ".blue(),
            "<k>".blue().bold(),
            " Quit ".blue(),
            "<Q> ".blue().bold(),
        ]));

        let block = Block::bordered()
            .border_style(Style::default().blue())
            .title(title.alignment(Alignment::Center))
            .title(
                instruction
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    // App::default() will init app with counter set to 0, and exit set to false
    let app_result = App::default().run(&mut terminal);
    if let Err(err) = tui::restore() {
        eprintln!(
            "Fialed to restore terminal. Run reset, or restart terimnal to fix.\n{}",
            err
        )
    }

    app_result
}
