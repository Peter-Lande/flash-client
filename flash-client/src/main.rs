use std::{cell::RefCell, io::Error};

use screen::{Screen, ScreenState};
use tui::widgets::ListState;

mod card;
mod screen;
mod util;

fn main() -> Result<(), Error> {
    let mut screen = Screen::new(ScreenState::LocalMenu(
        RefCell::new(ListState::default()),
        Vec::new().into_boxed_slice(),
        Vec::new().into_boxed_slice(),
    ))
    .expect("Terminal could not be created.");
    screen.init().expect("Terminal could not initialize.");
    screen.run().expect("Terminal failed during content loop.");
    screen.exit().expect("Terminal failed on deinitialization.");
    Ok(())
}
