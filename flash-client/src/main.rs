use std::io::Error;

use screen::{Screen, ScreenState};

mod card;
mod screen;
mod util;

fn main() -> Result<(), Error> {
    let mut screen = Screen::new(ScreenState::LocalMenu).expect("Terminal could not be created.");
    screen.run().expect("Terminal failed during content loop.");
    Ok(())
}
