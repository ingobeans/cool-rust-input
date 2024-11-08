use crossterm::event::{ self, Event, KeyCode };
use crossterm::{ execute, cursor, terminal, style::{ Color, SetForegroundColor, ResetColor } };
use std::io::{ self, Write, stdout };
use std::time::Duration;
use std::cmp;

pub trait CustomInput {
    fn handle_key_press(&mut self, key: &Event) -> bool {
        false
    }
    fn before_draw_text(&mut self, terminal_size: (u16, u16)) {}
    fn after_draw_text(&mut self, terminal_size: (u16, u16)) {}
    fn get_offset(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        (0, 0)
    }
    fn get_size(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        terminal_size
    }
}
pub struct DefaultInput;
impl CustomInput for DefaultInput {}

pub struct CoolInput<H: CustomInput> {
    pub text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub listening: bool,
    pub custom_input: H,
}

impl<H: CustomInput> CoolInput<H> {
    pub fn new(handler: H) -> Self {
        CoolInput {
            text: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            listening: false,
            custom_input: handler,
        }
    }
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        self.update_text()?;
        self.update_cursor()?;
        Ok(())
    }
    fn update_cursor(&mut self) -> Result<(), std::io::Error> {
        let terminal_size = terminal::size()?;
        let (offset_x, offset_y) = self.custom_input.get_offset(terminal_size);
        execute!(
            stdout(),
            cursor::MoveTo((self.cursor_x as u16) + offset_x, (self.cursor_y as u16) + offset_y)
        )?;
        Ok(())
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
    fn remove_character(&mut self, x: usize, y: usize) -> Result<(), std::io::Error> {
        let mut new = String::new();
        let mut cur_x = 0;
        let mut cur_y = 0;

        if x == 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.text
                .lines()
                .nth(self.cursor_y)
                .ok_or_else(||
                    std::io::Error::new(std::io::ErrorKind::Other, "Cursor at invalid position")
                )?
                .chars()
                .count();
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
        Ok(())
    }
    fn update_text(&mut self) -> Result<(), std::io::Error> {
        let terminal_size = terminal::size()?;
        let (width, height) = self.custom_input.get_size(terminal_size);
        let (offset_x, offset_y) = self.custom_input.get_offset(terminal_size);
        self.custom_input.before_draw_text(terminal_size);
        let lines = self.text.lines().count();

        for y in 0..height {
            let y_line_index = y.checked_sub(offset_y);
            if y_line_index.is_some() {
                let y_line_index = y_line_index.unwrap();
                if y_line_index < (lines as u16) {
                    let line = self.text
                        .lines()
                        .nth(y_line_index as usize)
                        .ok_or_else(||
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Cursor at invalid position"
                            )
                        )?;
                    print!(
                        "\x1b[{};0H{}",
                        y + 1,
                        String::from(" ").repeat(offset_x as usize) +
                            &String::from(line) +
                            &" ".repeat((width - (line.chars().count() as u16)).into())
                    );
                } else {
                    print!("\x1b[{};0H{}", y + 1, " ".repeat(width as usize));
                }
            }
        }

        io::stdout().flush()?;
        self.custom_input.after_draw_text(terminal_size);
        Ok(())
    }
    pub fn handle_key_press(&mut self, key: Event) -> Result<(), std::io::Error> {
        if self.custom_input.handle_key_press(&key) {
            return Ok(());
        }
        match key {
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            self.insert_string(c, self.cursor_x, self.cursor_y);
                            self.cursor_x += 1;
                            self.update_text()?;
                            self.update_cursor()?;
                        }
                        KeyCode::Enter => {
                            self.insert_string('\n', self.cursor_x, self.cursor_y);
                            self.cursor_x = 0;
                            self.cursor_y += 1;
                            self.update_text()?;
                            self.update_cursor()?;
                        }
                        KeyCode::Backspace => {
                            if self.cursor_x > 0 || self.cursor_y != 0 {
                                self.remove_character(self.cursor_x, self.cursor_y)?;
                                self.update_text()?;
                                self.update_cursor()?;
                            }
                        }
                        KeyCode::Esc => {
                            self.listening = false;
                        }
                        KeyCode::Up => {
                            if self.cursor_y > 0 {
                                self.cursor_y -= 1;
                                self.cursor_x = cmp::min(
                                    self.text
                                        .lines()
                                        .nth(self.cursor_y)
                                        .ok_or_else(||
                                            std::io::Error::new(
                                                std::io::ErrorKind::Other,
                                                "Cursor at invalid position"
                                            )
                                        )?
                                        .chars()
                                        .count(),
                                    self.cursor_x
                                );
                            }
                            self.update_cursor()?;
                        }
                        KeyCode::Down => {
                            if self.text.lines().count() > 0 {
                                if self.cursor_y < self.text.lines().count() - 1 {
                                    self.cursor_y += 1;
                                    self.cursor_x = cmp::min(
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .ok_or_else(||
                                                std::io::Error::new(
                                                    std::io::ErrorKind::Other,
                                                    "Cursor at invalid position"
                                                )
                                            )?
                                            .chars()
                                            .count(),
                                        self.cursor_x
                                    );
                                    self.update_cursor()?;
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
                                        .ok_or_else(||
                                            std::io::Error::new(
                                                std::io::ErrorKind::Other,
                                                "Cursor at invalid position"
                                            )
                                        )?
                                        .chars()
                                        .count();
                                }
                            }
                            self.update_cursor()?;
                        }
                        KeyCode::Right => {
                            if self.text.lines().count() > 0 {
                                if
                                    self.cursor_y != self.text.lines().count() - 1 ||
                                    self.cursor_x <
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .ok_or_else(||
                                                std::io::Error::new(
                                                    std::io::ErrorKind::Other,
                                                    "Cursor at invalid position"
                                                )
                                            )?
                                            .chars()
                                            .count()
                                {
                                    if
                                        self.cursor_x !=
                                        self.text
                                            .lines()
                                            .nth(self.cursor_y)
                                            .ok_or_else(||
                                                std::io::Error::new(
                                                    std::io::ErrorKind::Other,
                                                    "Cursor at invalid position"
                                                )
                                            )?
                                            .chars()
                                            .count()
                                    {
                                        self.cursor_x += 1;
                                    } else {
                                        self.cursor_y += 1;
                                        self.cursor_x = 0;
                                    }
                                    self.update_cursor()?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub fn listen(&mut self) -> Result<(), std::io::Error> {
        let terminal_size = terminal::size()?;
        let (offset_x, offset_y) = self.custom_input.get_offset(terminal_size);

        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            SetForegroundColor(Color::Blue),
            cursor::MoveTo((self.cursor_x as u16) + offset_x, (self.cursor_y as u16) + offset_y)
        )?;
        self.render()?;
        self.listening = true;
        while self.listening {
            if event::poll(Duration::from_millis(50))? {
                self.handle_key_press(event::read()?)?;
            }
        }
        execute!(
            stdout(),
            ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
}
