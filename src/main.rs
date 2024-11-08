use cool_rust_input::{ CoolInput, NoneCustomKeyPressHandler, CustomKeyPressHandler };
use crossterm::event::{ Event, KeyCode };

// example custom key press handler
// that disables b key
pub struct CoolCustomKeyPressHandler;
impl CustomKeyPressHandler for CoolCustomKeyPressHandler {
    fn handle_key_press(&mut self, key: &crossterm::event::Event) -> bool {
        if let Event::Key(key_event) = key {
            if let KeyCode::Char(c) = key_event.code {
                if c == 'b' {
                    return true;
                }
            }
        }
        return false;
    }
}

fn main() {
    //let mut cool_input = CoolInput::new(CoolCustomKeyPressHandler);
    let mut cool_input = CoolInput::new(NoneCustomKeyPressHandler);
    cool_input.listen().unwrap();
}
