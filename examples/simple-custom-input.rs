use cool_rust_input::{
    set_terminal_line, CoolInput, CustomInputHandler, HandlerContext, InputTransform,
};
use crossterm::{
    queue,
    style::{Color, SetForegroundColor},
};
use std::io::stdout;

struct MyHandler;
impl CustomInputHandler for MyHandler {
    fn get_input_transform(&mut self, ctx: HandlerContext) -> InputTransform {
        let size = (ctx.terminal_size.0 - 10, ctx.terminal_size.1 - 2);
        let offset = (5, 2);
        InputTransform { size, offset }
    }
    fn after_draw_text(&mut self, _: HandlerContext) {
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
