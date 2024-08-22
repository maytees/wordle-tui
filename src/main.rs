mod tui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Paragraph, Widget,
    },
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Result},
};

use rand::seq::SliceRandom;

#[derive(Debug, Default)]
pub struct App {
    target_word: String,
    // This is a way to store the old rounds
    // u8 is the round, and the string would be split using chars(),
    // "distributed" to the boxes
    store: HashMap<u8, String>,
    // 0,1,2,3, or 4 (basically the rows (just add 1 to index))
    // Default: 0
    validation_store: HashMap<u8, String>,
    last_submitted_string: String,
    round: u8,
    reveal_answer: bool,
    typing: String,
    win: bool,
    loss: bool,
    // For errors such as "wrong word", and etc.
    error: String,
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

    pub fn check_word_exist(&self) -> bool {
        let path = "fiveletterwords.txt";
        let file = File::open(path).expect("Failed to open file!");
        let reader = BufReader::new(file);

        let words: Vec<String> = reader.lines().filter_map(Result::ok).collect();

        words.contains(&self.typing)
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
                if !self.win && !self.loss && self.typing.len() < 5 {
                    self.typing.push(char)
                }
            }
            KeyCode::Backspace => {
                if !self.win && !self.loss {
                    self.typing.pop();
                }
            }
            KeyCode::Enter => {
                if !self.win && !self.loss {
                    self.submit_word()
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn submit_word(&mut self) {
        if self.typing.len() != 5 {
            self.error = String::from("Word must be 5 letters long!");
            return;
        }

        // 1. Check if word even exists in the list
        if !self.check_word_exist() {
            self.error = String::from("Word doesn't exist!");
            return;
        }

        // 2. If the word exists, go character by character and validate
        // 3. If validation succeeds, set win = true
        // 4. If validation doesn't succeed,
        //      If the current round is == 4 (5th round)
        //      Exit for now.
        //      If the current round is <4, then increase the round
        //      & set typing = ""

        // X = not in word, Y = in word, not in right place, G = in the right place
        // For example: if you type MUSIC, but the word was MULCH
        //              it would be GGXXY
        let mut result = String::new();

        for (i, char) in self.typing.chars().enumerate() {
            if let Some(target_char) = self.target_word.chars().nth(i) {
                if target_char == char {
                    result.push('G');
                } else if self.target_word.contains(char) {
                    result.push('Y');
                } else {
                    result.push('X');
                }
            }
        }

        self.validation_store.insert(self.round, result.clone());
        self.last_submitted_string = self.typing.clone();

        if result == "GGGGG" {
            self.win = true;
        }

        self.store.insert(self.round, self.typing.clone());

        if self.round == 4 && !self.win {
            self.loss = true;
        } else if !self.win {
            self.round += 1;
            self.typing.clear();
        }

        self.error.clear();
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
        let title = Title::from(Line::styled(
            " Wordle TUI in Rust ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        let instruction = Title::from(Line::from(vec![
            Span::styled(
                " <Ctrl-r> to refresh word ",
                Style::default().fg(Color::Blue),
            ),
            Span::styled(
                " <Ctrl-v> to reveal word ",
                Style::default().fg(Color::Blue),
            ),
            Span::styled(" <Ctrl-q> to exit ", Style::default().fg(Color::Blue)),
        ]));

        let top_block = Block::bordered()
            .border_style(Style::default().fg(Color::Blue))
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

        // index of row_constraint = round
        // index of col_constraint = letter
        let row_constraint = vec![Constraint::Percentage(50); 5];
        let col_constraint = vec![Constraint::Percentage(50); 5];

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraint)
            .split(grid_area);

        let smaller_area = Rect::new(38, 4, 110, 40);

        for (row_index, row) in vertical_layout.iter().enumerate() {
            let horizontal_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraint.clone())
                .split(*row);

            for (i, cell) in horizontal_layout.iter().enumerate() {
                let cell_block = Block::bordered()
                    .border_style(Style::default().fg(Color::White))
                    .border_type(BorderType::Rounded);

                cell_block.render(*cell, buf);

                // Get the current char from the string for the current row from the store
                if let Some(value) = self.store.get(&(row_index as u8)) {
                    let current_char = value.chars().nth(i).unwrap_or(' ');
                    let color = if self.win && value == &self.last_submitted_string {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else if current_char == self.target_word.chars().nth(i).unwrap_or(' ') {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else if self.target_word.contains(current_char) {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    };

                    BigText::builder()
                        .pixel_size(PixelSize::Quadrant)
                        .lines(vec![Line::styled(current_char.to_string(), color)])
                        .centered()
                        .build()
                        .render(*cell, buf);

                    Block::default().render(smaller_area, buf);
                }

                if let Some(c) = self.typing.chars().nth(i) {
                    if row_index == self.round as usize {
                        let style = Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD);
                        BigText::builder()
                            .pixel_size(PixelSize::Quadrant)
                            .lines(vec![Line::styled(c.to_string(), style)])
                            .centered()
                            .build()
                            .render(*cell, buf);

                        Block::default().render(smaller_area, buf);
                    }
                }
            }
        }

        if self.reveal_answer {
            let current_word_text = Text::styled(
                self.target_word.as_str(),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::UNDERLINED),
            );
            let word_area = centered_rect(30, 95, outer_layout[0]);
            Paragraph::new(current_word_text)
                .alignment(Alignment::Center)
                .render(word_area, buf);
        }

        let error_text = Text::styled(
            self.error.as_str(),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
        );
        let err_area = centered_rect(30, 90, outer_layout[0]);
        Paragraph::new(error_text)
            .alignment(Alignment::Center)
            .render(err_area, buf);

        if self.win {
            let win_text = Text::styled(
                "You won!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );
            let win_area = centered_rect(30, 85, outer_layout[0]);
            Paragraph::new(win_text)
                .alignment(Alignment::Center)
                .render(win_area, buf);
        } else if self.loss {
            let loss_text = Text::styled(
                format!("You lost! The word was: {}", self.target_word),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            );
            let loss_area = centered_rect(30, 85, outer_layout[0]);
            Paragraph::new(loss_text)
                .alignment(Alignment::Center)
                .render(loss_area, buf);
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
