use std::io::stdout;
use crossterm::{
    execute, queue, terminal, cursor, Result, 
    event::{self, Event, MouseEvent, MouseEventKind, KeyEvent, KeyCode},
};


fn start() -> Result<()> {
    execute!(stdout(), 
        terminal::EnterAlternateScreen, 
        event::EnableMouseCapture, cursor::Hide);

    loop {
        match event::read() {
            Ok(Event::Mouse(MouseEvent{kind : MouseEventKind::Down(e), .. })) => 
                println!("Good: {:?}", e),
            Ok(Event::Key(KeyEvent{code : KeyCode::Esc, ..})) => break,
            _ => {}
        }
    }

    execute!(stdout(), 
        terminal::LeaveAlternateScreen,    
        event::DisableMouseCapture, cursor::Show);

    Ok(())
}

fn main() {
    match start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e)
    }
}
