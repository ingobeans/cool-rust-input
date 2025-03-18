use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseEventKind,
};
use crossterm::{
    cursor, execute, queue,
    style::ResetColor,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::cmp;
use std::io::Result;
use std::io::{self, stdout, Write};

// Get slice of string by starting character index and end character index
fn get_slice_of_string(text: &str, start: usize, end: usize) -> String {
    let mut new_text = String::new();
    let length = text.chars().count();

    for i in start..end {
        if i >= length {
            break;
        }
        new_text.insert(
            new_text.len(),
            text.chars().nth(i).expect("Char at pos should exist"),
        );
    }
    new_text
}

/// Helper function to draw text to the screen by a coordinate
pub fn set_terminal_line(text: &str, x: usize, y: usize, overwrite: bool) -> Result<()> {
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

/// A basic default input handler that implements all default functions of the [CustomInputHandler] trait.
pub struct DefaultInputHandler;
impl CustomInputHandler for DefaultInputHandler {}

/// Returned by [CustomInputHandler's handle_key_press](CustomInputHandler::handle_key_press) to signal how the key event should be handled.
pub enum KeyPressResult {
    /// Tells the input that this event has been handled, and shouldn't be further processed.
    Handled,
    /// Tells the input to stop, like it is finished / submitted.
    Stop,
    /// Continue handling event as normal.
    Continue,
}

/// Context given to [CustomInputHandler]
pub struct HandlerContext<'a> {
    pub text_data: &'a mut TextInputData,
    pub terminal_size: &'a (u16, u16),
}

/// Struct for size and offset of an [input](CoolInput)
pub struct InputTransform {
    pub size: (u16, u16),
    pub offset: (u16, u16),
}

/// Trait that allows custom implementations / behaviour of an [input](CoolInput)
#[allow(unused_variables)]
pub trait CustomInputHandler {
    /// Called before handling of every key press.
    fn handle_key_press(&mut self, key: &Event, ctx: HandlerContext) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if key_event.kind == KeyEventKind::Press {
                // Make pressing Escape stop the input
                if let KeyCode::Esc = key_event.code {
                    return KeyPressResult::Stop;
                }

                // Make CTRL + C also stop
                if let KeyCode::Char(c) = key_event.code {
                    if c == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        return KeyPressResult::Stop;
                    }
                }
            }
        }
        KeyPressResult::Continue
    }
    /// Called before the user's text input is drawn. Here you can ex. change color of the inputted text
    fn before_draw_text(&mut self, ctx: HandlerContext) {
        let _ = queue!(stdout(), ResetColor);
    }
    /// Called after the user's text is drawn. Here you can ex. draw other text like information or a title of the document.
    fn after_draw_text(&mut self, ctx: HandlerContext) {}
    /// Called after the cursor is updated/drawn. Here you can ex. disable cursor blinking or hide it all together
    fn after_update_cursor(&mut self, ctx: HandlerContext) {}
    /// Called by the parent [input](CoolInput) to get the input area's size and offset (in a [InputTransform]).
    fn get_input_transform(&mut self, ctx: HandlerContext) -> InputTransform {
        let size = *ctx.terminal_size;
        let offset = (0, 0);
        InputTransform { size, offset }
    }
}

/// Handles key presses, writing text, and moving the cursor
pub struct TextInputData {
    pub text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub tab_width: usize,
}

/// The main input type. Uses a custom input handler (a struct which implements [CustomInputHandler])
pub struct CoolInput<H: CustomInputHandler> {
    pub text_data: TextInputData,
    pub scroll_x: usize,
    pub scroll_y: usize,
    pub listening: bool,
    pub custom_input: H,
}

impl TextInputData {
    pub fn write_char(&mut self, c: char) -> Result<()> {
        self.insert_char(c, self.cursor_x, self.cursor_y);
        self.move_cursor_right()?;
        Ok(())
    }
    pub fn insert_char(&mut self, c: char, x: usize, y: usize) {
        let mut new = String::new();
        let mut cur_x = 0;
        let mut cur_y = 0;

        if x == 0 && y == 0 {
            self.text.insert(0, c);
        } else {
            for char in self.text.chars() {
                cur_x += 1;
                if char == '\n' {
                    cur_y += 1;
                    cur_x = 0;
                }
                new.insert(new.len(), char);
                if cur_x == x && cur_y == y {
                    new.insert(new.len(), c);
                }
            }
            self.text = new;
        }
    }
    pub fn remove_character(&mut self, x: usize, y: usize) -> Result<()> {
        let mut new = String::new();
        let mut cur_x = 0;
        let mut cur_y = 0;

        if x == 0 {
            self.move_cursor_up()?;
            self.cursor_x = self.get_current_line_length()?;
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
    fn move_cursor_end(&mut self) -> Result<()> {
        if self.get_amt_lines() > 0 {
            self.cursor_x = self.get_current_line_length()?;
        }
        Ok(())
    }
    fn move_cursor_up(&mut self) -> Result<()> {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
        } else {
            self.cursor_x = 0;
        }
        Ok(())
    }
    fn move_cursor_down(&mut self) -> Result<()> {
        if self.cursor_y < self.get_amt_lines() - 1 {
            self.cursor_y += 1;
            self.cursor_x = cmp::min(self.get_current_line_length()?, self.cursor_x);
        } else {
            self.move_cursor_end()?;
        }
        Ok(())
    }
    fn move_cursor_left(&mut self) -> Result<()> {
        if self.cursor_x > 0 || self.cursor_y != 0 {
            if self.cursor_x > 0 {
                self.cursor_x -= 1;
            } else {
                self.cursor_y -= 1;
                self.cursor_x = self.get_current_line_length()?;
            }
        }
        Ok(())
    }
    fn move_cursor_right(&mut self) -> Result<()> {
        if self.cursor_y != self.get_amt_lines() - 1
            || self.cursor_x < self.get_current_line_length()?
        {
            if self.cursor_x != self.get_current_line_length()? {
                self.cursor_x += 1;
            } else {
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
        }

        Ok(())
    }
    pub fn get_amt_lines(&mut self) -> usize {
        let mut amt = self.text.lines().count();
        if self.text.ends_with("\n") {
            amt += 1;
        }
        amt
    }
    pub fn get_line_at(&mut self, y: usize) -> Option<&str> {
        if self.text.ends_with("\n") && y == self.text.lines().count() {
            return Some("");
        }
        self.text.lines().nth(y)
    }
    pub fn get_current_line_length(&mut self) -> Result<usize> {
        let line = self.get_line_at(self.cursor_y);
        match line {
            Some(text) => Ok(text.chars().count()),
            None => Err(std::io::Error::new(
                io::ErrorKind::Other,
                "Couldn't get length of current line because it doesn't exist.",
            )),
        }
    }
    fn handle_key_press(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char(c) => {
                self.insert_char(c, self.cursor_x, self.cursor_y);
                self.move_cursor_right()?;
            }
            KeyCode::Enter => {
                self.insert_char('\n', self.cursor_x, self.cursor_y);
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            KeyCode::Backspace => {
                if self.cursor_x > 0 || self.cursor_y != 0 {
                    self.remove_character(self.cursor_x, self.cursor_y)?;
                }
            }
            KeyCode::Tab => {
                for _ in 0..self.tab_width {
                    self.insert_char(' ', self.cursor_x, self.cursor_y);
                }
                self.cursor_x += self.tab_width;
            }
            KeyCode::Delete => {
                if self.get_amt_lines() > 0 {
                    let line_length = self.get_current_line_length()?;
                    if self.cursor_x < line_length || self.cursor_y != self.get_amt_lines() - 1 {
                        if self.cursor_x == line_length {
                            self.cursor_x = 0;
                            self.cursor_y += 1;
                        } else {
                            self.cursor_x += 1;
                        }
                        self.remove_character(self.cursor_x, self.cursor_y)?;
                    }
                }
            }
            KeyCode::Up => {
                self.move_cursor_up()?;
            }
            KeyCode::Down => {
                if self.get_amt_lines() > 0 {
                    self.move_cursor_down()?;
                }
            }
            KeyCode::Left => {
                self.move_cursor_left()?;
            }
            KeyCode::Right if self.get_amt_lines() > 0 => {
                self.move_cursor_right()?;
            }
            KeyCode::Home => {
                self.cursor_x = 0;
            }
            KeyCode::End => {
                self.move_cursor_end()?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl<H: CustomInputHandler> CoolInput<H> {
    pub fn new(handler: H, tab_width: usize) -> Self {
        CoolInput {
            text_data: TextInputData {
                text: String::new(),
                cursor_x: 0,
                cursor_y: 0,
                tab_width,
            },
            listening: false,
            scroll_x: 0,
            scroll_y: 0,
            custom_input: handler,
        }
    }
    /// Get the size of the terminal running the program
    pub fn get_terminal_size(&self) -> Result<(u16, u16)> {
        let mut terminal_size = terminal::size()?;
        terminal_size.1 -= 1;
        Ok(terminal_size)
    }
    pub fn get_input_transform(&mut self) -> Result<InputTransform> {
        let terminal_size = self.get_terminal_size()?;
        let input_transform = self.custom_input.get_input_transform(HandlerContext {
            text_data: &mut self.text_data,
            terminal_size: &terminal_size,
        });
        let mut size = input_transform.size;
        let offset = input_transform.offset;
        if size.0 + offset.0 > terminal_size.0 {
            size.0 = terminal_size.0.saturating_sub(offset.0);
        }
        if size.1 + offset.1 > terminal_size.1 {
            size.1 = terminal_size.1.saturating_sub(offset.1);
        }
        Ok(InputTransform { size, offset })
    }
    /// Render all text and update cursor
    pub fn render(&mut self) -> Result<()> {
        self.update_text()?;
        self.update_cursor()?;
        io::stdout().flush()?;
        Ok(())
    }
    fn update_cursor(&mut self) -> Result<()> {
        if !self.cursor_within_screen()? {
            queue!(stdout(), cursor::Hide)?;
            return Ok(());
        }
        let terminal_size = self.get_terminal_size()?;
        let input_transform = self.get_input_transform()?;

        let x =
            self.text_data.cursor_x as i16 + input_transform.offset.0 as i16 - self.scroll_x as i16;
        let x: u16 = cmp::max(x, 0_i16) as u16;
        let x = cmp::min(x, input_transform.offset.0 + input_transform.size.0);
        let target_y = (self.text_data.cursor_y as u16) + input_transform.offset.1;
        let target_y = target_y.saturating_sub(self.scroll_y as u16);
        let y = cmp::min(
            cmp::min(
                target_y,
                input_transform.offset.1 + input_transform.size.1 - 1,
            ),
            terminal_size.1 - 1,
        );
        queue!(stdout(), cursor::Show)?;
        queue!(stdout(), cursor::MoveTo(x, y))?;

        self.custom_input.after_update_cursor(HandlerContext {
            text_data: &mut self.text_data,
            terminal_size: &terminal_size,
        });
        Ok(())
    }
    fn update_text(&mut self) -> Result<()> {
        let terminal_size = self.get_terminal_size()?;
        let input_transform = self.get_input_transform()?;

        self.custom_input.before_draw_text(HandlerContext {
            text_data: &mut self.text_data,
            terminal_size: &terminal_size,
        });

        let offset_y = input_transform.offset.1 as i16;
        for y in offset_y..offset_y + (input_transform.size.1 as i16) {
            let y_line_index = y - offset_y + (self.scroll_y as i16);
            if y_line_index >= 0 && y_line_index < (self.text_data.text.lines().count() as i16) {
                if let Some(line) = self.text_data.get_line_at(y_line_index as usize) {
                    let text = get_slice_of_string(
                        line,
                        self.scroll_x,
                        self.scroll_x + input_transform.size.0 as usize,
                    );
                    set_terminal_line(&text, input_transform.offset.0 as usize, y as usize, true)?;
                }
            } else {
                set_terminal_line("", input_transform.offset.0 as usize, y as usize, true)?;
            }
        }

        self.custom_input.after_draw_text(HandlerContext {
            text_data: &mut self.text_data,
            terminal_size: &terminal_size,
        });

        Ok(())
    }
    fn scroll_in_view(&mut self, moving_right: bool, moving_down: bool) -> Result<()> {
        let input_transform = self.get_input_transform()?;
        self.scroll_x = self.keep_scroll_axis_in_view(
            self.scroll_x,
            self.text_data.cursor_x,
            input_transform.size.0 as usize,
            moving_right,
        );
        self.scroll_y = self.keep_scroll_axis_in_view(
            self.scroll_y,
            self.text_data.cursor_y,
            input_transform.size.1 as usize,
            moving_down,
        );
        Ok(())
    }
    fn keep_scroll_axis_in_view(
        &mut self,
        mut scroll_amt: usize,
        cursor_pos: usize,
        bounds: usize,
        moving_direction: bool,
    ) -> usize {
        if moving_direction {
            if cursor_pos > bounds - 1 {
                scroll_amt = cmp::max(scroll_amt, cursor_pos - bounds + 1);
            }
        } else if cursor_pos < scroll_amt {
            scroll_amt = cursor_pos;
        }
        scroll_amt
    }
    pub fn cursor_within_screen(&mut self) -> Result<bool> {
        let input_transform = self.get_input_transform()?;
        let height = input_transform.size.1;

        let screen_starts_y = self.scroll_y;
        let screen_ends_y = self.scroll_y + height as usize;

        let cursor_pos_y = self.text_data.cursor_y;

        let show = screen_starts_y <= cursor_pos_y && cursor_pos_y < screen_ends_y;

        Ok(show)
    }
    /// Handle an event
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        let terminal_size = self.get_terminal_size()?;
        let old_cursor_x = self.text_data.cursor_x;
        let old_cursor_y = self.text_data.cursor_y;
        match self.custom_input.handle_key_press(
            &event,
            HandlerContext {
                text_data: &mut self.text_data,
                terminal_size: &terminal_size,
            },
        ) {
            KeyPressResult::Handled => {
                self.scroll_in_view(
                    self.text_data.cursor_x > old_cursor_x,
                    self.text_data.cursor_y > old_cursor_y,
                )?;
                self.render()?;
                return Ok(());
            }
            KeyPressResult::Stop => {
                self.listening = false;
                return Ok(());
            }
            KeyPressResult::Continue => match event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        self.text_data.handle_key_press(key_event)?;
                        self.scroll_in_view(
                            self.text_data.cursor_x > old_cursor_x,
                            self.text_data.cursor_y > old_cursor_y,
                        )?;
                        self.render()?;
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::ScrollUp => {
                        self.scroll_y = self.scroll_y.saturating_sub(1);
                        self.render()?;
                    }
                    MouseEventKind::ScrollDown => {
                        let input_transform = self.get_input_transform()?;
                        let content_ends_y = self.text_data.text.split('\n').count() as u16
                            + input_transform.offset.1;
                        let (_, height) = self.get_terminal_size()?;
                        let screen_ends_y = height + self.scroll_y as u16;
                        if screen_ends_y <= content_ends_y {
                            self.scroll_y += 1;
                            self.render()?;
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
        }
        Ok(())
    }
    /// Start listening for key presses without preparing the terminal
    pub fn listen_quiet(&mut self) -> Result<()> {
        self.listening = true;
        while self.listening {
            self.handle_event(event::read()?)?;
        }
        Ok(())
    }
    /// Prepare the terminal for input
    pub fn pre_listen(&mut self) -> Result<()> {
        let input_transform = self.get_input_transform()?;
        enable_raw_mode()?;
        execute!(
            stdout(),
            EnableMouseCapture,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(
                (self.text_data.cursor_x as u16) + input_transform.offset.0,
                (self.text_data.cursor_y as u16) + input_transform.offset.1
            )
        )?;
        Ok(())
    }
    /// Restore the terminal after input is finished.
    pub fn post_listen(&mut self) -> Result<()> {
        execute!(
            stdout(),
            ResetColor,
            DisableMouseCapture,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
        )?;
        disable_raw_mode()?;
        Ok(())
    }
    /// Prepare terminal and start to listen for key presses until finished.
    pub fn listen(&mut self) -> Result<()> {
        self.pre_listen()?;
        self.render()?;
        self.listen_quiet()?;
        self.post_listen()?;
        Ok(())
    }
}
