use cool_rust_input::{set_terminal_line, CoolInput, CustomInput};
use crossterm::{
    queue,
    style::{Color, SetForegroundColor},
};
use std::io::stdout;

struct MyHandler;
impl CustomInput for MyHandler {
    fn get_offset(&mut self, _: (u16, u16), _: String) -> (u16, u16) {
        (5, 2)
    }
    fn get_size(&mut self, terminal_size: (u16, u16), _: String) -> (u16, u16) {
        (terminal_size.0 - 10, terminal_size.1 - 5)
    }
    fn after_draw_text(&mut self, _: (u16, u16), _: String) {
        // we'll use this function to display a title text

        let _ = queue!(stdout(), SetForegroundColor(Color::Green));
        let _ = set_terminal_line("[MY COOL TEXT EDITOR PROGRAM]", 5, 0, true);
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut my_input = CoolInput::new(MyHandler, 0);
    my_input.listen()?;
    Ok(())
}
