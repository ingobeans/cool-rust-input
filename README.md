# cool-rust-input

![image](https://github.com/user-attachments/assets/e9c93489-037e-4aaa-a0ab-395db5e8ee68)

an input crate for fine control over each key press and rendering of text. by default allows multiline input, but custom behaviour can be added to make enter submit.

cross platform, tested on windows 11, arch and termux.

## basic code sample

```rust
use cool_rust_input::{CoolInput, DefaultInputHandler};

fn main() -> Result<(), std::io::Error> {
    let mut my_input = CoolInput::new(DefaultInputHandler, 0);
    my_input.listen()?;
    Ok(())
}
```

## custom handler sample

```rust
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
```

## todo:

- markdown support (to some degree) (maybe)
- pgdown/pgup
- ctrl + left/right arrow
