use cool_rust_input::{ CoolInput, DefaultInput, CustomInput, set_terminal_line };
use crossterm::event::{ Event, KeyCode };

pub struct CoolCustomInput;
impl CustomInput for CoolCustomInput {
    fn handle_key_press(&mut self, key: &crossterm::event::Event) -> bool {
        if let Event::Key(key_event) = key {
            if let KeyCode::Char(c) = key_event.code {
                if c == 'q' {
                    return true;
                }
            }
        }
        return false;
    }
    fn after_draw_text(&mut self, terminal_size: (u16, u16)) {
        println!("\x1b[1;0H    Welcome to my cool text editor. Here you can write cool stuff!");
        println!("\x1b[2;0H    Rules: ");
        println!("\x1b[3;0H        None!!");
        println!("\x1b[4;0H    {}", String::from("_").repeat((terminal_size.0 as usize) - 10));

        set_terminal_line(
            "Welcome to my cool text editor. Here you can write cool stuff!",
            5,
            0
        ).unwrap();
        set_terminal_line("Rules:", 5, 1).unwrap();
        set_terminal_line("None!!", 10, 2).unwrap();
        set_terminal_line(
            &String::from("_").repeat((terminal_size.0 as usize) - 10),
            5,
            3
        ).unwrap();
    }
    fn get_offset(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        (5, 5)
    }
    fn get_size(&mut self, terminal_size: (u16, u16)) -> (u16, u16) {
        (terminal_size.0 - 10, 5)
    }
}

fn main() {
    let mut cool_input = CoolInput::new(CoolCustomInput);
    //let mut cool_input = CoolInput::new(DefaultInput);
    cool_input.listen().unwrap();
}
