use crossterm::event::{ self, Event, KeyCode };
use crossterm::{ execute, cursor, terminal, style::{ Color, SetForegroundColor, ResetColor } };
use std::io::{ self, Write, stdout };
use std::time::Duration;
use std::cmp;

pub struct CoolInput {
    pub text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub listening: bool,
}

impl Default for CoolInput {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            listening: false,
        }
    }
}

impl CoolInput {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn render(&mut self) {
        self.update_text();
        self.update_cursor();
    }
    fn update_cursor(&mut self) {
        execute!(
            stdout(),
            cursor::MoveTo(self.cursor_x.try_into().unwrap(), self.cursor_y.try_into().unwrap())
        ).unwrap();
    }
    fn insert_string(&mut self, c: char, x: usize, y: usize) {
        let mut new = String::new();
        let mut cur_x = 0;
        let mut cur_y = 0;

        if x == 0 && y == 0 {
            self.text.insert(0, c);
        } else {
            let mut found = false;
            for char in self.text.chars() {
                cur_x += 1;
                if char == '\n' {
                    cur_y += 1;
                    cur_x = 0;
                }
                new.insert(new.len(), char);
                if cur_x == x && cur_y == y {
                    new.insert(new.len(), c);
                    found = true;
                }
            }
            if !found {
                println!("{}, {}", x, y);
                std::process::exit(1);
            }
            self.text = new;
        }
    }
    fn remove_character(&mut self, x: usize, y: usize) {
        let mut new = String::new();
        let mut cur_x = 0;
        let mut cur_y = 0;

        if x == 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.text.lines().nth(self.cursor_y).unwrap().chars().count();
        } else {
            self.cursor_x -= 1;
        }

        if self.text.len() > 0 {
            for char in self.text.chars() {
                cur_x += 1;
                if char == '\n' {
                    cur_y += 1;
                    cur_x = 0;
                }
                if cur_x != x || cur_y != y {
                    new.insert(new.len(), char);
                }
            }
        }
        self.text = new;
    }
    fn update_text(&mut self) {
        let size = terminal::size().unwrap();
        let height = size.1;
        let width = size.0;
        let lines = self.text.lines().count();

        for y in 0..height {
            if y < (lines as u16) {
                let line = self.text
                    .lines()
                    .nth(y as usize)
                    .unwrap();
                print!(
                    "\x1b[{};0H{}",
                    y + 1,
                    String::from(line) + &" ".repeat((width - (line.chars().count() as u16)).into())
                );
            } else {
                print!("\x1b[{};0H{}", y + 1, " ".repeat(width as usize));
            }
        }

        io::stdout().flush().unwrap();
    }
    pub fn on_key_press(&mut self, key: Event) {
        match key {
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            self.insert_string(c, self.cursor_x, self.cursor_y);
                            self.cursor_x += 1;
                            self.update_text();
                            self.update_cursor();
                        }
                        KeyCode::Enter => {
                            self.insert_string('\n', self.cursor_x, self.cursor_y);
                            self.cursor_x = 0;
                            self.cursor_y += 1;
                            self.update_text();
                            self.update_cursor();
                        }
                        KeyCode::Backspace => {
                            if self.cursor_x > 0 || self.cursor_y != 0 {
                                self.remove_character(self.cursor_x, self.cursor_y);
                                self.update_text();
                                self.update_cursor();
                            }
                        }
                        KeyCode::Esc => {
                            self.listening = false;
                        }
                        KeyCode::Up => {
                            if self.cursor_y > 0 {
                                self.cursor_y -= 1;
                                self.cursor_x = cmp::min(
                                    self.text.lines().nth(self.cursor_y).unwrap().chars().count(),
                                    self.cursor_x
                                );
                            }
                            self.update_cursor();
                        }
                        KeyCode::Down => {
                            if self.text.lines().count() > 0 {
                                if self.cursor_y < self.text.lines().count() - 1 {
                                    self.cursor_y += 1;
                                    self.cursor_x = cmp::min(
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .unwrap()
                                            .chars()
                                            .count(),
                                        self.cursor_x
                                    );
                                    self.update_cursor();
                                }
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor_x > 0 || self.cursor_y != 0 {
                                if self.cursor_x > 0 {
                                    self.cursor_x -= 1;
                                } else {
                                    self.cursor_y -= 1;
                                    self.cursor_x = self.text
                                        .lines()
                                        .nth(self.cursor_y)
                                        .unwrap()
                                        .chars()
                                        .count();
                                }
                            }
                            self.update_cursor();
                        }
                        KeyCode::Right => {
                            if self.text.lines().count() > 0 {
                                if
                                    self.cursor_y != self.text.lines().count() - 1 ||
                                    self.cursor_x <
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .unwrap()
                                            .chars()
                                            .count()
                                {
                                    if
                                        self.cursor_x !=
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .unwrap()
                                            .chars()
                                            .count()
                                    {
                                        self.cursor_x += 1;
                                    } else {
                                        self.cursor_y += 1;
                                        self.cursor_x = 0;
                                    }
                                    self.update_cursor();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }
    pub fn listen(&mut self) {
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            SetForegroundColor(Color::Blue),
            cursor::MoveTo(self.cursor_x.try_into().unwrap(), self.cursor_y.try_into().unwrap())
        ).unwrap();
        self.listening = true;
        while self.listening {
            if event::poll(Duration::from_millis(50)).unwrap() {
                self.on_key_press(event::read().unwrap());
            }
        }
        execute!(
            stdout(),
            ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        ).unwrap();
    }
}
