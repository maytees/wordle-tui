mod tui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Padding, Paragraph, Widget,
    },
    Frame,
};
use tui_big_text::{BigText, PixelSize};

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
    typing: String,
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
            KeyCode::Char(char) => {
                if self.typing.len() != 5 {
                    self.typing.push(char)
                }
            }
            KeyCode::Backspace => if let Some(_) = self.typing.pop() {},
            _ => {}
        };

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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
            .border_type(BorderType::Rounded)
            .title(title.alignment(Alignment::Center))
            .title(
                instruction
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(area);

        top_block.render(outer_layout[0], buf);

        let grid_area = centered_rect(50, 80, outer_layout[0]);

        let row_constraint = vec![Constraint::Percentage(50); 5];
        let col_constraint = vec![Constraint::Percentage(50); 5];

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraint)
            .split(grid_area);

        for (row_index, row) in vertical_layout.iter().enumerate() {
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraint.clone())
                .split(*row);

            for (i, cell) in horizontal_layout.iter().enumerate() {
                let cell_block = Block::bordered()
                    .border_style(Style::default().yellow())
                    .border_type(BorderType::Rounded);

                cell_block.render(*cell, buf);

                if let Some(c) = self.typing.chars().nth(i) {
                    if row_index == self.round as usize {
                        let cell_block = Block::bordered()
                            .border_style(Style::default().yellow())
                            .border_type(BorderType::Thick);

                        BigText::builder()
                            .pixel_size(PixelSize::Quadrant)
                            .style(Style::new().blue())
                            .lines(vec![Line::from(c.to_string().bold().gray())])
                            .centered()
                            .build()
                            .render(*cell, buf);

                        cell_block.render(area, buf)
                    }
                }
            }

            if self.reveal_answer {
                let current_word_text = Text::from(self.target_word.as_str().red().underlined());
                let word_area = centered_rect(30, 30, outer_layout[0]);
                Paragraph::new(current_word_text)
                    .centered()
                    .render(word_area, buf);
            }
            // let current_word_text = Text::from(vec![Line::from(vec![if self.reveal_answer {
            //     self.target_word.as_str().red().underlined()
            // } else {
            //     "".into()
            // }])]);

            // let inner_block = Block::bordered()
            //     .title(Title::from("W".bold().red()))
            //     .border_style(Style::default().red())
            //     .border_type(BorderType::Rounded);

            // let outer_layout = Layout::default()
            //     .direction(Direction::Vertical)
            //     .constraints([Constraint::Percentage(100)])
            //     .split(area);

            // top_block.render(outer_layout[0], buf);

            // let inner_area = centered_rect(30, 20, outer_layout[0]);

            // Paragraph::new(current_word_text)
            //     .centered()
            //     .block(inner_block)
            //     .render(area, buf);
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    // App::default() will init app with counter set to 0, and exit set to false
    let _ = App::default().run(&mut terminal);
    let _ = tui::restore();

    Ok(())
}
