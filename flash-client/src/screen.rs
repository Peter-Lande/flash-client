use std::{cell::RefCell, env::current_exe, error::Error, io::stdout, rc::Rc, time::Duration};

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};

use crate::util;
#[derive(Clone)]
pub enum ScreenState {
    //The field represents the state of the deck cursor
    LocalMenu,
}

pub struct Screen {
    state: Rc<ScreenState>,
    local_menu_state: Rc<RefCell<ListState>>,
    local_decks: Box<[String]>,
}

impl Screen {
    pub fn new(state: ScreenState) -> Result<Self, Box<dyn Error>> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        return Ok(Screen {
            state: Rc::new(state),
            local_menu_state: Rc::new(RefCell::new(list_state)),
            local_decks: Screen::get_current_local_decks().into_boxed_slice(),
        });
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        loop {
            let initial_state = self.state.clone();
            if poll(Duration::from_millis(200))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => match *initial_state {
                            ScreenState::LocalMenu => {
                                let new_state = util::offset_state(
                                    &self.local_menu_state.clone().borrow(),
                                    1,
                                    true,
                                    self.local_decks.len() - 1,
                                );
                                self.local_menu_state = Rc::new(RefCell::new(new_state));
                            }
                        },
                        KeyCode::Up => match *initial_state {
                            ScreenState::LocalMenu => {
                                let new_state = util::offset_state(
                                    &self.local_menu_state.clone().borrow(),
                                    1,
                                    false,
                                    self.local_decks.len() - 1,
                                );
                                self.local_menu_state = Rc::new(RefCell::new(new_state));
                            }
                        },
                        _ => (),
                    }
                }
            }
            // Gets the approriate references, builds content for the screen, then draws to stdout.
            let header = self.build_header();
            let middle_panel_content = self.build_main_panel_content();
            let footer = self.build_footer();
            let menu_layout = Screen::build_layout(&mut terminal.get_frame());
            match *self.state.clone() {
                ScreenState::LocalMenu => terminal.draw(|f| {
                    f.render_widget(header, menu_layout[0]);
                    f.render_widget(footer, menu_layout[4]);
                    f.render_stateful_widget(
                        middle_panel_content,
                        menu_layout[2],
                        //I know this is improper but its the only way to make it work...
                        &mut (*self.local_menu_state).borrow_mut(),
                    );
                })?,
            };
        }
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn build_header(&self) -> Tabs<'static> {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let titles = vec![
                    Spans::from(Span::raw("Local")),
                    Spans::from(Span::raw("Remote")),
                ];
                Tabs::new(titles)
                    .block(
                        Block::default()
                            .title(" Flash ")
                            .borders(Borders::TOP | Borders::BOTTOM)
                            .title_alignment(Alignment::Center),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                    .select(0)
            }
        }
    }

    fn build_footer(&self) -> Paragraph<'static> {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let text = vec![Spans::from(vec![
                    Span::raw("Selected '"),
                    Span::raw(
                        self.local_decks[self.local_menu_state.borrow().selected().unwrap()]
                            .clone(),
                    ),
                    Span::raw("' "),
                    Span::raw("Navigate("),
                    Span::raw("↑/↓) "),
                    Span::raw("(q)uit"),
                ])];
                return Paragraph::new(text)
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                    .alignment(Alignment::Left);
            }
        }
    }

    fn build_layout<B: Backend>(f: &mut tui::Frame<B>) -> Vec<Rect> {
        let first_layer_layout = Layout::default()
            .direction(Direction::Vertical)
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(f.size());
        let header_section = first_layer_layout[0];
        let content_section = first_layer_layout[1];
        let footer_section = first_layer_layout[2];
        let second_layer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(content_section);
        return vec![
            header_section,
            second_layer_layout[0],
            second_layer_layout[1],
            second_layer_layout[2],
            footer_section,
        ];
    }

    fn build_main_panel_content(&self) -> List<'static> {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let mut list_items: Vec<ListItem> = self
                    .local_decks
                    .iter()
                    .map(|x| ListItem::new(x.to_owned()))
                    .collect();
                return List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
            }
        }
    }
    fn get_current_local_decks() -> Vec<String> {
        let mut cur_dir = current_exe().expect("Could not find path to executable.");
        cur_dir.pop();
        cur_dir.push("decks");
        cur_dir.push("local/");
        let mut list_items_str =
            util::get_sub_directories(cur_dir.as_path()).expect("Failed to read directories.");
        list_items_str.sort();
        list_items_str.push(String::from("Add new deck..."));
        list_items_str
    }
}
