use crossterm::event::{self, Event, KeyCode};
use crossterm::{
    cursor, execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::cmp;
use std::io::{self, stdout, Write};

pub enum KeyPressResult {
    Handled,
    Stop,
    Continue,
}

#[allow(unused_variables)]
pub trait CustomInput {
    fn handle_key_press(&mut self, key: &Event, current_text: String) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if let KeyCode::Esc = key_event.code {
                return KeyPressResult::Stop;
            }
        }
        KeyPressResult::Continue
    }
    fn before_draw_text(&mut self, terminal_size: (u16, u16), current_text: String) {
        let _ = execute!(stdout(), SetForegroundColor(Color::Blue));
    }
    fn after_draw_text(&mut self, terminal_size: (u16, u16), current_text: String) {
        let _ = execute!(stdout(), ResetColor);
    }
    fn get_offset(&mut self, terminal_size: (u16, u16), current_text: String) -> (u16, u16) {
        (0, 0)
    }
    fn get_size(&mut self, terminal_size: (u16, u16), current_text: String) -> (u16, u16) {
        terminal_size
    }
}
pub struct DefaultInput;
impl CustomInput for DefaultInput {}

pub struct CoolInput<H: CustomInput> {
    pub text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_y: usize,
    pub listening: bool,
    pub custom_input: H,
    pub tab_width: usize,
}

pub fn set_terminal_line(
    text: &str,
    x: usize,
    y: usize,
    overwrite: bool,
) -> Result<(), std::io::Error> {
    execute!(stdout(), cursor::Hide)?;
    if overwrite {
        print!("\x1b[{};{}H\x1b[2K{}", y + 1, x + 1, text);
    } else {
        print!("\x1b[{};{}H{}", y + 1, x + 1, text);
    }
    Ok(())
}

impl<H: CustomInput> CoolInput<H> {
    pub fn new(handler: H, tab_width: usize) -> Self {
        CoolInput {
            text: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            listening: false,
            scroll_y: 0,
            tab_width: tab_width,
            custom_input: handler,
        }
    }
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        self.update_text()?;
        self.update_cursor()?;
        Ok(())
    }
    fn update_cursor(&mut self) -> Result<(), std::io::Error> {
        execute!(stdout(), cursor::Show)?;
        let terminal_size = self.get_terminal_size()?;
        let (width, height) = self
            .custom_input
            .get_size(terminal_size, self.text.to_string());
        let (offset_x, offset_y) = self
            .custom_input
            .get_offset(terminal_size, self.text.to_string());
        let x = cmp::min((self.cursor_x as u16) + offset_x, offset_x + width);
        let target_y = (self.cursor_y as u16) + offset_y;
        let target_y = target_y.checked_sub(self.scroll_y as u16).unwrap_or(0);
        let y = cmp::min(
            cmp::min(target_y, offset_y + height - 1),
            terminal_size.1 - 1,
        );
        execute!(stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }
    pub fn get_terminal_size(&mut self) -> Result<(u16, u16), std::io::Error> {
        let mut terminal_size = terminal::size()?;
        terminal_size.1 -= 1;
        Ok(terminal_size)
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
            self.move_cursor_up()?;
            self.cursor_x = self.get_current_line_length()?;
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
        let terminal_size = self.get_terminal_size()?;
        let (_width, height) = self
            .custom_input
            .get_size(terminal_size, self.text.to_string());
        let (offset_x, offset_y) = self
            .custom_input
            .get_offset(terminal_size, self.text.to_string());
        self.custom_input
            .before_draw_text(terminal_size, self.text.to_string());
        let offset_y = offset_y as i16;
        for y in offset_y..offset_y + (height as i16) {
            let y_line_index = y - offset_y + (self.scroll_y as i16);
            if y_line_index >= 0 && y_line_index < (self.text.lines().count() as i16) {
                let line = self.get_line_at(y_line_index as usize)?;
                set_terminal_line(&line, offset_x as usize, y as usize, true)?;
            } else {
                set_terminal_line("", offset_x as usize, y as usize, true)?;
            }
        }
        self.custom_input
            .after_draw_text(terminal_size, self.text.to_string());
        io::stdout().flush()?;
        Ok(())
    }
    fn move_cursor_end(&mut self) -> Result<(), std::io::Error> {
        if self.get_amt_lines() > 0 {
            self.cursor_x = self.get_current_line_length()?;
            self.update_cursor()?;
        }
        Ok(())
    }
    fn move_cursor_up(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
            if self.cursor_y < self.scroll_y {
                self.scroll_y -= 1;
            }
            self.update_text()?;
        } else {
            self.cursor_x = 0;
        }
        Ok(())
    }
    fn move_cursor_down(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_y < self.get_amt_lines() - 1 {
            self.cursor_y += 1;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
            let text = self.text.to_string();
            let terminal_size = self.get_terminal_size()?;
            let input_height = self.custom_input.get_size(terminal_size, text).1;
            if self.cursor_y >= (input_height as usize) + self.scroll_y {
                self.scroll_y += 1;
            }
            self.update_text()?;
        } else {
            self.move_cursor_end()?;
        }
        Ok(())
    }
    fn get_amt_lines(&mut self) -> usize {
        let mut amt = self.text.lines().count();
        if self.text.ends_with("\n") {
            amt += 1;
        }
        amt
    }
    fn get_line_at(&mut self, y: usize) -> Result<String, std::io::Error> {
        Ok(self.text.lines().nth(y).unwrap_or("").to_string())
    }
    fn get_current_line_length(&mut self) -> Result<usize, std::io::Error> {
        Ok(self.get_line_at(self.cursor_y)?.chars().count())
    }
    pub fn handle_key_press(&mut self, key: Event) -> Result<(), std::io::Error> {
        match self
            .custom_input
            .handle_key_press(&key, self.text.to_string())
        {
            KeyPressResult::Handled => {
                return Ok(());
            }
            KeyPressResult::Stop => {
                self.listening = false;
                return Ok(());
            }
            KeyPressResult::Continue => match key {
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
                                self.cursor_y += 1;
                                self.cursor_x = 0;
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
                            KeyCode::Tab => {
                                for _ in 0..self.tab_width {
                                    self.insert_string(' ', self.cursor_x, self.cursor_y);
                                }
                                self.cursor_x += self.tab_width;
                                self.update_text()?;
                                self.update_cursor()?;
                            }
                            KeyCode::Delete => {
                                if self.get_amt_lines() > 0 {
                                    let line_length = self.get_current_line_length()?;
                                    if self.cursor_x < line_length
                                        || self.cursor_y != self.get_amt_lines() - 1
                                    {
                                        if self.cursor_x == line_length {
                                            self.cursor_x = 0;
                                            self.cursor_y += 1;
                                        } else {
                                            self.cursor_x += 1;
                                        }
                                        self.remove_character(self.cursor_x, self.cursor_y)?;
                                        self.update_text()?;
                                        self.update_cursor()?;
                                    }
                                }
                            }
                            KeyCode::Up => {
                                self.move_cursor_up()?;
                                self.update_cursor()?;
                            }
                            KeyCode::Down => {
                                if self.get_amt_lines() > 0 {
                                    self.move_cursor_down()?;
                                    self.update_cursor()?;
                                }
                            }
                            KeyCode::Left => {
                                if self.cursor_x > 0 || self.cursor_y != 0 {
                                    if self.cursor_x > 0 {
                                        self.cursor_x -= 1;
                                    } else {
                                        self.cursor_y -= 1;
                                        self.cursor_x = self.get_current_line_length()?;
                                    }
                                }
                                self.update_text()?;
                                self.update_cursor()?;
                            }
                            KeyCode::Right => {
                                if self.get_amt_lines() > 0 {
                                    if self.cursor_y != self.get_amt_lines() - 1
                                        || self.cursor_x < self.get_current_line_length()?
                                    {
                                        if self.cursor_x != self.get_current_line_length()? {
                                            self.cursor_x += 1;
                                        } else {
                                            self.cursor_y += 1;
                                            self.cursor_x = 0;
                                        }
                                        self.update_text()?;
                                        self.update_cursor()?;
                                    }
                                }
                            }
                            KeyCode::Home => {
                                self.cursor_x = 0;
                                self.update_text()?;
                                self.update_cursor()?;
                            }
                            KeyCode::End => {
                                self.update_text()?;
                                self.move_cursor_end()?;
                            }
                            _ => {}
                        }
                    }
                }
                _ => (),
            },
        }
        Ok(())
    }
    pub fn listen_quiet(&mut self) -> Result<(), std::io::Error> {
        self.listening = true;
        while self.listening {
            self.handle_key_press(event::read()?)?;
        }
        Ok(())
    }
    pub fn pre_listen(&mut self) -> Result<(), std::io::Error> {
        let terminal_size = self.get_terminal_size()?;
        let (offset_x, offset_y) = self
            .custom_input
            .get_offset(terminal_size, self.text.to_string());
        enable_raw_mode()?;
        execute!(
            stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(
                (self.cursor_x as u16) + offset_x,
                (self.cursor_y as u16) + offset_y
            )
        )?;
        Ok(())
    }
    pub fn post_listen(&mut self) -> Result<(), std::io::Error> {
        execute!(
            stdout(),
            ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        disable_raw_mode()?;
        Ok(())
    }
    pub fn listen(&mut self) -> Result<(), std::io::Error> {
        self.pre_listen()?;
        self.render()?;
        self.listen_quiet()?;
        self.post_listen()?;
        Ok(())
    }
}
