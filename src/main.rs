use cool_rust_input::{ CoolInput, DefaultInput, CustomInput };
use crossterm::event::{ Event, KeyCode };

pub struct CoolCustomInput;
impl CustomInput for CoolCustomInput {
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
    fn before_draw_text(&mut self, terminal_size: (u16, u16)) {
        println!("\x1b[1;0H    Welcome to my cool text editor. Here you can write cool stuff!");
        println!("\x1b[2;0H    Rules: ");
        println!("\x1b[3;0H        None!!");
        println!("\x1b[4;0H    {}", String::from("_").repeat((terminal_size.0 as usize) - 10));
    }
    fn after_draw_text(&mut self, terminal_size: (u16, u16)) {}
    fn get_offset(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        (5, 5)
    }
    fn get_size(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        (terminal_size.0 - 5, terminal_size.1)
    }
}

fn main() {
    let mut cool_input = CoolInput::new(CoolCustomInput);
    //let mut cool_input = CoolInput::new(DefaultInput);
    cool_input.listen().unwrap();
}
