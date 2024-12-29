# cool-rust-input

![image](https://github.com/user-attachments/assets/e9c93489-037e-4aaa-a0ab-395db5e8ee68)

an input crate for fine control over each key press and rendering of text. by default allows multiline input, but custom behaviour can be added to make enter submit.

cross platform, tested on windows 11, arch and termux.

## basic code sample

```rust
use cool_rust_input::{CoolInput, DefaultInputHandler};

fn main() -> Result<(), std::io::Error>{
	let mut my_input = CoolInput::new(DefaultInputHandler, 0);
	my_input.listen()?;
	Ok(())
}
```

## custom handler sample

```rust
use cool_rust_input::{CoolInput, CustomInput, set_terminal_line};

struct MyHandler;
impl CustomInput for MyHandler {
	fn get_offset(&mut self, _: (u16, u16), _: String) -> (u16, u16) {
		(5,2)
	}
	fn get_size(&mut self, terminal_size: (u16, u16), _: String) -> (u16, u16) {
		(terminal_size.0 - 10, terminal_size.1 - 5)
	}
	fn after_draw_text(&mut self, _: (u16, u16), _: String) {
		// we'll use this function to display a title text

		let color = "\x1b[1;31m"; // make title red with ansi escape codes
		let text = format!("{color}hello and welcome");
		let _ = set_terminal_line(&text,5,0,true);
	}
}

fn main() -> Result<(), std::io::Error>{
	let mut my_input = CoolInput::new(MyHandler, 0);
	my_input.listen()?;
    Ok(())
}
```

## todo:

- markdown support (to some degree) (maybe)
- pgdown/pgup
- ctrl + left/right arrow
