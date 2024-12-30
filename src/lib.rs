use crossterm::event::{self, Event, KeyCode};
use crossterm::{
    cursor, execute, queue,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::cmp;
use std::io::{self, stdout, Write};

/// Returned by [CustomInput's handle_key_press](CustomInput::handle_key_press) to signal how the key event should be handled.
pub enum KeyPressResult {
    /// Tells the input that this event has been handled, and shouldn't be further processed.
    Handled,
    /// Tells the input to stop, like it is finished / submitted.
    Stop,
    /// Continue handling event as normal.
    Continue,
}

/// Trait that allows custom implementations / behaviour of an [input](CoolInput)
#[allow(unused_variables)]
pub trait CustomInput {
    /// Called before handling of every key press.
    fn handle_key_press(&mut self, key: &Event, current_text: String) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if let KeyCode::Esc = key_event.code {
                return KeyPressResult::Stop;
            }
        }
        KeyPressResult::Continue
    }
    /// Called before the user's text input is drawn. Here you can ex. change color of the inputted text
    fn before_draw_text(&mut self, terminal_size: (u16, u16), current_text: String) {
        let _ = queue!(stdout(), SetForegroundColor(Color::Blue));
    }
    /// Called after the user's text is drawn. Here you can ex. draw other text like information or a title of the document.
    fn after_draw_text(&mut self, terminal_size: (u16, u16), current_text: String) {
        let _ = queue!(stdout(), ResetColor);
    }
    /// Called by the parent [input](CoolInput) to get the input area's offset, I.e. where the user will start typing.
    fn get_offset(&mut self, terminal_size: (u16, u16), current_text: String) -> (u16, u16) {
        (0, 0)
    }
    /// Called by the parent [input](CoolInput) to get the input area's size
    fn get_size(&mut self, terminal_size: (u16, u16), current_text: String) -> (u16, u16) {
        terminal_size
    }
}
/// A basic default input handler that implements all default functions of the [CustomInput] trait.
pub struct DefaultInputHandler;
impl CustomInput for DefaultInputHandler {}

fn get_slice_of_string(text: String, start: usize, end: usize) -> String {
    let mut new_text = String::new();
    let length = text.chars().count();

    for i in start..end {
        if i >= length {
            break;
        }
        new_text.insert(
            new_text.chars().count(),
            text.chars()
                .nth(i)
                .expect("Char at pos should exist"),
        );
    }
    new_text
}

/// The main input type. Uses a custom input handler (a struct which implements [CustomInput])
pub struct CoolInput<H: CustomInput> {
    pub text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub listening: bool,
    pub custom_input: H,
    pub tab_width: usize,
}

/// Helper function to draw text to the screen by a coordinate
pub fn set_terminal_line(
    text: &str,
    x: usize,
    y: usize,
    overwrite: bool,
) -> Result<(), std::io::Error> {
    if overwrite {
        queue!(
            stdout(),
            cursor::MoveTo(x as u16, y as u16),
            terminal::Clear(terminal::ClearType::CurrentLine)
        )?;
        print!("{text}");
    } else {
        queue!(stdout(), cursor::MoveTo(x as u16, y as u16))?;
        print!("{text}");
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
            scroll_x: 0,
            scroll_y: 0,
            tab_width,
            custom_input: handler,
        }
    }
    /// Render all text and update cursor
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        self.update_text()?;
        self.update_cursor()?;
        io::stdout().flush()?;
        Ok(())
    }
    fn update_cursor(&mut self) -> Result<(), std::io::Error> {
        let terminal_size = self.get_terminal_size()?;
        let (width, height) = self
            .custom_input
            .get_size(terminal_size, self.text.to_string());
        let (offset_x, offset_y) = self
            .custom_input
            .get_offset(terminal_size, self.text.to_string());
        let x = self.cursor_x as i16 + offset_x as i16 - self.scroll_x as i16;
        let x: u16 = cmp::max(x, 0_i16) as u16;
        let x = cmp::min(x, offset_x + width);
        let target_y = (self.cursor_y as u16) + offset_y;
        let target_y = target_y.saturating_sub(self.scroll_y as u16);
        let y = cmp::min(
            cmp::min(target_y, offset_y + height - 1),
            terminal_size.1 - 1,
        );
        queue!(stdout(), cursor::Show)?;
        queue!(stdout(), cursor::MoveTo(x, y))?;
        Ok(())
    }
    /// Get the size of the terminal running the program
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
            self.keep_scroll_x_in_view(true)?;
        } else {
            self.move_cursor_left()?;
        }

        if !self.text.is_empty() {
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
        let (width, height) = self
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
                let text = get_slice_of_string(line, self.scroll_x, self.scroll_x + width as usize);
                set_terminal_line(&text, offset_x as usize, y as usize, true)?;
            } else {
                set_terminal_line("", offset_x as usize, y as usize, true)?;
            }
        }
        self.custom_input
            .after_draw_text(terminal_size, self.text.to_string());
        Ok(())
    }
    fn move_cursor_end(&mut self) -> Result<(), std::io::Error> {
        if self.get_amt_lines() > 0 {
            self.cursor_x = self.get_current_line_length()?;
            self.keep_scroll_x_in_view(true)?;
            self.update_cursor()?;
        }
        Ok(())
    }
    fn move_cursor_up(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let original_x = self.cursor_x;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
            if self.cursor_y < self.scroll_y {
                self.scroll_y -= 1;
            }

            self.keep_scroll_x_in_view(self.cursor_x >= original_x)?;
            self.render()?;
        } else {
            self.cursor_x = 0;
            self.scroll_x = 0;
        }
        Ok(())
    }
    fn move_cursor_down(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_y < self.get_amt_lines() - 1 {
            self.cursor_y += 1;
            let original_x = self.cursor_x;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
            let text = self.text.to_string();
            let terminal_size = self.get_terminal_size()?;
            let input_height = self.custom_input.get_size(terminal_size, text).1;
            if self.cursor_y >= (input_height as usize) + self.scroll_y {
                self.scroll_y += 1;
            }
            self.keep_scroll_x_in_view(self.cursor_x >= original_x)?;
            self.render()?;
        } else {
            self.move_cursor_end()?;
        }
        Ok(())
    }
    fn move_cursor_left(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_x > 0 || self.cursor_y != 0 {
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
                self.keep_scroll_x_in_view(false)?;
            } else {
                self.cursor_y -= 1;
                self.cursor_x = self.get_current_line_length()?;
                self.keep_scroll_x_in_view(true)?;
            }
        }
        Ok(())
    }
    fn keep_scroll_x_in_view(&mut self, moving_right: bool) -> Result<(), std::io::Error> {
        if moving_right {
            let text = self.text.to_string();
            let terminal_size = self.get_terminal_size()?;
            let input_width = self.custom_input.get_size(terminal_size, text).0;
            if self.cursor_x > input_width as usize - 1 {
                self.scroll_x = cmp::max(self.scroll_x, self.cursor_x - input_width as usize + 1);
            }
        } else if self.cursor_x < self.scroll_x {
            self.scroll_x = self.cursor_x;
        }
        Ok(())
    }
    fn move_cursor_right(&mut self) -> Result<(), std::io::Error> {
        if self.cursor_y != self.get_amt_lines() - 1
            || self.cursor_x < self.get_current_line_length()?
        {
            if self.cursor_x != self.get_current_line_length()? {
                self.cursor_x += 1;
                self.keep_scroll_x_in_view(true)?;
            } else {
                self.cursor_y += 1;
                self.cursor_x = 0;
                self.keep_scroll_x_in_view(false)?;
            }
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
    /// Handle a key event
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
            KeyPressResult::Continue => {
                if let Event::Key(key_event) = key {
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                self.insert_string(c, self.cursor_x, self.cursor_y);
                                self.move_cursor_right()?;
                                self.render()?;
                            }
                            KeyCode::Enter => {
                                self.insert_string('\n', self.cursor_x, self.cursor_y);
                                self.cursor_y += 1;
                                self.cursor_x = 0;
                                self.keep_scroll_x_in_view(false)?;
                                self.render()?;
                            }
                            KeyCode::Backspace => {
                                if self.cursor_x > 0 || self.cursor_y != 0 {
                                    self.remove_character(self.cursor_x, self.cursor_y)?;
                                    self.render()?;
                                }
                            }
                            KeyCode::Tab => {
                                for _ in 0..self.tab_width {
                                    self.insert_string(' ', self.cursor_x, self.cursor_y);
                                }
                                self.cursor_x += self.tab_width;
                                self.render()?;
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
                                        self.render()?;
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
                                self.move_cursor_left()?;
                                self.render()?;
                            }
                            KeyCode::Right if self.get_amt_lines() > 0 => {
                                self.move_cursor_right()?;
                                self.render()?;
                            }
                            KeyCode::Home => {
                                self.cursor_x = 0;
                                self.keep_scroll_x_in_view(false)?;
                                self.render()?;
                            }
                            KeyCode::End => {
                                self.move_cursor_end()?;
                                self.render()?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Start listening for key presses without preparing the terminal
    pub fn listen_quiet(&mut self) -> Result<(), std::io::Error> {
        self.listening = true;
        while self.listening {
            self.handle_key_press(event::read()?)?;
        }
        Ok(())
    }
    /// Prepare the terminal for input
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
    /// Restore the terminal after input is finished.
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
    /// Prepare terminal and start to listen for key presses until finished.
    pub fn listen(&mut self) -> Result<(), std::io::Error> {
        self.pre_listen()?;
        self.render()?;
        self.listen_quiet()?;
        self.post_listen()?;
        Ok(())
    }
}
