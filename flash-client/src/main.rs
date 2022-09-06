mod card;
mod screen;
mod util;

fn main() -> Result<(), std::io::Error> {
    let mut screen = screen::Screen::new(screen::ScreenState::LocalMenu(
        tui::widgets::ListState::default(),
        Vec::new().into_boxed_slice(),
        Vec::new().into_boxed_slice(),
    ))
    .expect("Terminal could not be created.");
    screen.init().expect("Terminal could not initialize.");
    screen.run().expect("Terminal failed during content loop.");
    screen.exit().expect("Terminal failed on deinitialization.");
    Ok(())
}
