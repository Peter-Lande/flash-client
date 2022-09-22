use std::{
    cell::RefCell,
    cmp::min,
    env::current_exe,
    error::Error,
    io::{stdout, Stdout},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

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

use crate::{deck::Deck, util};
#[derive(Clone)]
pub enum ScreenState {
    LocalMenu,
    DeckViewer,
    EditMode(Rc<ScreenState>),
}

pub struct Screen {
    state: Rc<ScreenState>,
    local_menu_state: Rc<RefCell<ListState>>,
    local_decks_names: Box<[String]>,
    current_deck: Rc<RefCell<Deck>>,
}

impl Screen {
    pub fn new(state: ScreenState) -> Result<Self, Box<dyn Error>> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        return Ok(Screen {
            state: Rc::new(state),
            local_menu_state: Rc::new(RefCell::new(list_state)),
            local_decks_names: Screen::get_current_local_decks().into_boxed_slice(),
            current_deck: Rc::new(RefCell::new(Deck::default())),
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
                                    self.local_decks_names.len() - 1,
                                );
                                self.local_menu_state = Rc::new(RefCell::new(new_state));
                            }
                            _ => (),
                        },
                        KeyCode::Up => match *initial_state {
                            ScreenState::LocalMenu => {
                                let new_state = util::offset_state(
                                    &self.local_menu_state.clone().borrow(),
                                    1,
                                    false,
                                    self.local_decks_names.len() - 1,
                                );
                                self.local_menu_state = Rc::new(RefCell::new(new_state));
                            }
                            _ => (),
                        },
                        KeyCode::Char('o') => match *initial_state {
                            ScreenState::LocalMenu => {
                                let mut cur_dir: PathBuf = match current_exe() {
                                    Ok(mut exe_dir) => {
                                        exe_dir.pop();
                                        exe_dir
                                    }
                                    Err(_) => PathBuf::from(std::env::var("HOME").unwrap()),
                                };
                                cur_dir.push("decks");
                                cur_dir.push("local");
                                cur_dir.push(
                                    &self.local_decks_names[self
                                        .local_menu_state
                                        .borrow()
                                        .selected()
                                        .unwrap_or_default()],
                                );
                                if let Ok(deck) = Deck::read_from_dir(&cur_dir) {
                                    self.current_deck = Rc::new(RefCell::new(deck));
                                    self.state = Rc::new(ScreenState::DeckViewer);
                                }
                            }
                            _ => (),
                        },
                        KeyCode::Right => match *initial_state {
                            ScreenState::DeckViewer => {
                                self.current_deck.borrow_mut().increment_deck();
                            }
                            _ => (),
                        },
                        KeyCode::Left => match *initial_state {
                            ScreenState::DeckViewer => {
                                self.current_deck.borrow_mut().decrement_deck();
                            }
                            _ => (),
                        },
                        KeyCode::Esc => match *initial_state {
                            ScreenState::DeckViewer => {
                                self.current_deck = Rc::new(RefCell::new(Deck::default()));
                                self.state = Rc::new(ScreenState::LocalMenu);
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            //First we find the areas of the screen we are drawing to, then we draw each part of the screen using the appropriate function.
            let menu_layout = Screen::build_layout(&mut terminal.get_frame());
            terminal.draw(|f| {
                self.render_header(f, &menu_layout[0]);
                self.render_footer(f, &menu_layout[4]);
                self.render_middle_panel_content(f, &menu_layout[2]);
            })?;
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

    fn render_header(&self, f: &mut tui::Frame<CrosstermBackend<Stdout>>, area: &Rect) -> () {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let titles = vec![
                    Spans::from(Span::raw("Local")),
                    Spans::from(Span::raw("Remote")),
                ];
                let header = Tabs::new(titles)
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
                    .select(0);
                f.render_widget(header, *area);
            }
            ScreenState::DeckViewer => {
                let titles = vec![Spans::from(vec![
                    Span::raw("Deck: "),
                    Span::raw(self.current_deck.borrow().deck_title.clone()),
                    Span::raw(" Progress: "),
                    Span::raw(
                        (min(
                            self.current_deck.borrow().cur_card.clone() + 1,
                            self.current_deck.borrow().len(),
                        ))
                        .to_string(),
                    ),
                    Span::raw("/"),
                    Span::raw((self.current_deck.borrow().len()).to_string()),
                ])];
                let header = Paragraph::new(titles)
                    .block(
                        Block::default()
                            .title(" Flash ")
                            .borders(Borders::TOP | Borders::BOTTOM)
                            .title_alignment(Alignment::Center),
                    )
                    .style(Style::default().fg(Color::White));
                f.render_widget(header, *area);
            }
            _ => (),
        };
    }

    fn render_footer(&self, f: &mut tui::Frame<CrosstermBackend<Stdout>>, area: &Rect) -> () {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let text = vec![Spans::from(vec![
                    Span::raw("Selected '"),
                    Span::raw(
                        self.local_decks_names[self.local_menu_state.borrow().selected().unwrap()]
                            .clone(),
                    ),
                    Span::raw("' "),
                    Span::raw("Navigate("),
                    Span::raw("↑/↓) "),
                    Span::raw("(o)pen "),
                    Span::raw("(q)uit"),
                ])];
                let footer = Paragraph::new(text)
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                    .alignment(Alignment::Left);
                f.render_widget(footer, *area);
            }
            ScreenState::DeckViewer => {
                let text = vec![Spans::from(vec![
                    Span::raw("Next Section/Next Card "),
                    Span::raw("(←/→) "),
                    Span::raw("(q)uit"),
                ])];
                let footer = Paragraph::new(text)
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                    .alignment(Alignment::Left);
                f.render_widget(footer, *area);
            }
            _ => (),
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

    fn render_middle_panel_content(
        &self,
        f: &mut tui::Frame<CrosstermBackend<Stdout>>,
        area: &Rect,
    ) -> () {
        match *self.state.clone() {
            ScreenState::LocalMenu => {
                let list_items: Vec<ListItem> = self
                    .local_decks_names
                    .iter()
                    .map(|x| ListItem::new(x.to_owned()))
                    .collect();
                let middle_panel = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
                //The state needs this weird configuration to work, sadly just how it is.
                f.render_stateful_widget(
                    middle_panel,
                    *area,
                    &mut (*self.local_menu_state).borrow_mut(),
                );
            }
            ScreenState::DeckViewer => {
                f.render_widget(self.current_deck.borrow().as_widget(), *area);
            }
            _ => (),
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
