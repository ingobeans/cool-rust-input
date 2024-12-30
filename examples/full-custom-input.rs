use cool_rust_input::{set_terminal_line, CoolInput, CustomInput, KeyPressResult};
use crossterm::event::{Event, KeyCode};
use crossterm::{
    queue,
    style::{Color, SetForegroundColor},
};
use std::io::stdout;

pub struct CoolCustomInput;
impl CustomInput for CoolCustomInput {
    fn handle_key_press(&mut self, key: &Event, _current_text: String) -> KeyPressResult {
        if let Event::Key(key_event) = key {
            if key_event.kind == crossterm::event::KeyEventKind::Press {
                // Make escape stop the input
                if let KeyCode::Esc = key_event.code {
                    return KeyPressResult::Stop;
                }
                // Disallow the user pressing the character 'S'
                if let KeyCode::Char(c) = key_event.code {
                    if c == 's' || c == 'S' {
                        return KeyPressResult::Handled;
                    }
                }
            }
        }
        KeyPressResult::Continue
    }
    fn before_draw_text(&mut self, _terminal_size: (u16, u16), _current_text: String) {
        let _ = queue!(stdout(), SetForegroundColor(Color::Green));
    }
    fn after_draw_text(&mut self, terminal_size: (u16, u16), current_text: String) {
        let _ = queue!(stdout(), SetForegroundColor(Color::White));
        set_terminal_line(
            "Welcome to my cool text editor. Here you can write cool stuff! Press ESC to exit.",
            5,
            0,
            true,
        )
        .unwrap();
        set_terminal_line("Rules:", 5, 1, true).unwrap();
        set_terminal_line("No typing the letter S", 10, 2, true).unwrap();

        let width = self.get_size(terminal_size, current_text).0;
        set_terminal_line(&String::from("_").repeat(width as usize), 5, 3, true).unwrap();
    }
    fn get_offset(&mut self, _terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (5, 5)
    }
    fn get_size(&mut self, terminal_size: (u16, u16), _current_text: String) -> (u16, u16) {
        (terminal_size.0 - 10, terminal_size.1 - 5)
    }
}

fn main() {
    let mut cool_input = CoolInput::new(CoolCustomInput, 0);
    cool_input.listen().unwrap();
}
