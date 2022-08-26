mod card;
mod screen;
mod util;

fn main() -> Result<(), std::io::Error> {
    let mut screen = screen::Screen::new(screen::ScreenState::LocalMenu(0))
        .expect("Terminal could not be created.");
    screen.init().expect("Terminal could not initialize.");
    screen.run().expect("Terminal failed during content loop.");
    screen.exit().expect("Terminal failed on deinitialization.");
    Ok(())
}
