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

## todo:

- markdown support (to some degree) (maybe)
- pgdown/pgup
- ctrl + left/right arrow
