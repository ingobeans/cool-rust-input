use cool_rust_input::{CoolInput, DefaultInputHandler};

fn main() -> Result<(), std::io::Error> {
    let mut my_input = CoolInput::new(DefaultInputHandler, 0);
    my_input.listen()?;
    Ok(())
}
