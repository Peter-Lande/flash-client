use std::{
    cell::RefCell,
    env::current_exe,
    error::Error,
    io::{stdout, Stdout},
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

use crate::util;
#[derive(Clone)]
pub enum ScreenState {
    //The field represents the state of the deck cursor
    LocalMenu(RefCell<ListState>, Box<[String]>, Box<[Rect]>),
}

pub struct Screen {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: Rc<ScreenState>,
}

impl Screen {
    pub fn new(state: ScreenState) -> Result<Self, Box<dyn Error>> {
        let temp_term = Terminal::new(CrosstermBackend::new(stdout()))?;
        return Ok(Screen {
            terminal: temp_term,
            state: Rc::new(state),
        });
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        execute!(
            self.terminal_mut().backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        match *self.get_state() {
            ScreenState::LocalMenu(..) => {
                let mut list_state = ListState::default();
                list_state.select(Some(0));
                self.state = Rc::new(ScreenState::LocalMenu(
                    RefCell::new(list_state),
                    Screen::get_current_local_decks().into_boxed_slice(),
                    Screen::build_layout(&mut self.terminal_mut().get_frame()).into_boxed_slice(),
                ))
            }
        }
        return Ok(());
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let initial_state = self.get_state();
            if poll(Duration::from_millis(200))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => match &*initial_state {
                            ScreenState::LocalMenu(list_state, decks, content_regions) => {
                                let new_state = util::offset_state(
                                    &list_state.borrow(),
                                    1,
                                    true,
                                    decks.len() - 1,
                                );
                                self.state = Rc::new(ScreenState::LocalMenu(
                                    RefCell::new(new_state),
                                    decks.to_owned(),
                                    content_regions.to_owned(),
                                ));
                            }
                        },
                        KeyCode::Up => match &*initial_state {
                            ScreenState::LocalMenu(list_state, decks, content_regions) => {
                                let new_state = util::offset_state(
                                    &list_state.borrow(),
                                    1,
                                    false,
                                    decks.len() - 1,
                                );
                                self.state = Rc::new(ScreenState::LocalMenu(
                                    RefCell::new(new_state),
                                    decks.to_owned(),
                                    content_regions.to_owned(),
                                ));
                            }
                        },
                        _ => (),
                    }
                }
            }
            // Gets the approriate references, builds content for the screen, then draws to stdout.
            let state = self.get_state();
            let terminal = self.terminal_mut();
            let header = Screen::build_header(state.clone());
            let middle_panel_content = Screen::build_main_panel_content(state.clone());
            let footer = Screen::build_footer(state.clone());
            match &*state {
                ScreenState::LocalMenu(list_state, _, content_regions) => terminal.draw(|f| {
                    f.render_widget(header, content_regions[0]);
                    f.render_widget(footer, content_regions[4]);
                    f.render_stateful_widget(
                        middle_panel_content,
                        content_regions[2],
                        //I know this is improper but its the only way to make it work...
                        &mut *list_state.borrow_mut(),
                    );
                })?,
            };
        }
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal_mut().backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal_mut().show_cursor()?;
        Ok(())
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        return &mut self.terminal;
    }

    fn get_state(&self) -> Rc<ScreenState> {
        return self.state.clone();
    }

    fn build_header(state: Rc<ScreenState>) -> Tabs<'static> {
        match *state {
            ScreenState::LocalMenu(..) => {
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

    fn build_footer(state: Rc<ScreenState>) -> Paragraph<'static> {
        match &*state.clone() {
            ScreenState::LocalMenu(list_state, decks, _) => {
                let text = vec![Spans::from(vec![
                    Span::raw("Selected '"),
                    Span::raw(decks[list_state.borrow().selected().unwrap()].clone()),
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

    fn build_main_panel_content(state: Rc<ScreenState>) -> List<'static> {
        match &*state.clone() {
            ScreenState::LocalMenu(_, current_decks, _) => {
                let mut list_items: Vec<ListItem> = current_decks
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
