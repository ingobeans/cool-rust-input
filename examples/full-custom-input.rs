use cool_rust_input::{
    set_terminal_line, CoolInput, CustomInputHandler, HandlerContext, InputTransform,
    KeyPressResult,
};
use crossterm::event::{Event, KeyCode};
use crossterm::{
    queue,
    style::{Color, SetForegroundColor},
};
use std::io::stdout;

pub struct CoolCustomInput;
impl CustomInputHandler for CoolCustomInput {
    fn handle_key_press(&mut self, key: &Event, _: HandlerContext) -> KeyPressResult {
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
    fn before_draw_text(&mut self, _: HandlerContext) {
        let _ = queue!(stdout(), SetForegroundColor(Color::Green));
    }
    fn after_draw_text(&mut self, ctx: HandlerContext) {
        let _ = queue!(stdout(), SetForegroundColor(Color::White));
        let _ = set_terminal_line(
            "Welcome to my cool text editor. Here you can write cool stuff! Press ESC to exit.",
            5,
            0,
            true,
        );
        let _ = set_terminal_line("Rules:", 5, 1, true);
        let _ = set_terminal_line("No typing the letter S", 10, 2, true);

        let width = self.get_input_transform(ctx).size.0;
        let _ = set_terminal_line(&String::from("_").repeat(width as usize), 5, 3, true);
    }
    fn get_input_transform(&mut self, ctx: HandlerContext) -> InputTransform {
        let size = (ctx.terminal_size.0 - 10, ctx.terminal_size.1 - 5);
        let offset = (5, 5);
        InputTransform { size, offset }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut cool_input = CoolInput::new(CoolCustomInput, 0);
    cool_input.listen()?;
    Ok(())
}
