use cool_rust_input::{
    set_terminal_line, CoolInput, CustomInputHandler, HandlerContext, KeyPressResult,
};
use crossterm::{
    event::{Event, KeyCode},
    queue,
    style::{Color, SetForegroundColor},
};
use std::io::stdout;

struct MyHandler;
impl CustomInputHandler for MyHandler {
    fn get_offset(&mut self, _: HandlerContext) -> (u16, u16) {
        (1, 2)
    }
    fn get_size(&mut self, ctx: HandlerContext) -> (u16, u16) {
        (ctx.terminal_size.0 - 1, ctx.terminal_size.1 - 2)
    }
    fn after_draw_text(&mut self, _: HandlerContext) {
        // we'll use this function to display a title text

        let _ = queue!(stdout(), SetForegroundColor(Color::Green));
        let _ = set_terminal_line("hello and welcome to my command prompt!", 0, 0, true);
        let _ = set_terminal_line(">", 0, 2, false);
    }
    fn handle_key_press(&mut self, key: &Event, _: HandlerContext) -> KeyPressResult {
        // Make Enter stop the input
        if let Event::Key(key_event) = key {
            if key_event.kind == crossterm::event::KeyEventKind::Press {
                if let KeyCode::Enter = key_event.code {
                    return KeyPressResult::Stop;
                }
            }
        }
        KeyPressResult::Continue
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut my_input = CoolInput::new(MyHandler, 0);
    my_input.listen()?;
    println!("your input was: '{}'", my_input.text_data.text);
    Ok(())
}
