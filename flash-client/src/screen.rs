use std::{
    cell::RefCell,
    cmp::min,
    error::Error,
    fs,
    io::{stdout, Stdout},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use ini::Ini;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};

use crate::{card::Card, deck::Deck, util};
#[derive(Clone)]
pub enum ScreenState {
    LocalMenu,
    DeckViewer,
    DeckEditor,
}

#[derive(Clone)]
pub enum EditMode {
    EditMenu(Rc<RefCell<ListState>>),
    AddItem,
    EditContent,
    None,
}

pub struct ScreenOptions {
    local_directory: PathBuf,
}

impl ScreenOptions {
    pub fn new(local_path: &str) -> Self {
        return ScreenOptions {
            local_directory: PathBuf::from(local_path),
        };
    }
}

pub struct Screen {
    state: Rc<ScreenState>,
    local_menu_state: Rc<RefCell<ListState>>,
    edit_menu_state: Rc<RefCell<ListState>>,
    local_decks_names: Box<[String]>,
    current_deck: Rc<RefCell<Deck>>,
    edit_mode: Rc<EditMode>,
    edit_failed: bool,
    right_panel_text_field: Rc<String>,
    options: Rc<ScreenOptions>,
}

impl Screen {
    pub fn new(state: ScreenState) -> Result<Self, Box<dyn Error>> {
        let mut local_list_state = ListState::default();
        local_list_state.select(Some(0));
        let mut edit_list_state = ListState::default();
        edit_list_state.select(Some(0));
        if let Some(mut config_dir) = home_dir() {
            config_dir.push(".flashrust");
            let mut local_dir = config_dir.clone();
            if let Ok(_) = fs::create_dir_all(&config_dir) {
                let mut config = Ini::new();
                local_dir.push("decks");
                local_dir.push("local");
                fs::create_dir_all(&local_dir)?;
                config
                    .with_section(Some("Setup"))
                    .set("local_dir", local_dir.as_os_str().to_str().unwrap());
                config_dir.push("config.ini");
                config.write_to_file(&config_dir)?;
            }
            let config = Ini::load_from_file(config_dir)?;
            if let Some(setup) = config.section(Some("Setup")) {
                if let Some(local_path) = setup.get("local_dir") {
                    let screen_options = ScreenOptions::new(local_path);
                    return Ok(Screen {
                        state: Rc::new(state),
                        local_menu_state: Rc::new(RefCell::new(local_list_state)),
                        edit_menu_state: Rc::new(RefCell::new(edit_list_state)),
                        local_decks_names: Screen::get_current_local_decks(&screen_options)
                            .into_boxed_slice(),
                        current_deck: Rc::new(RefCell::new(Deck::default())),
                        edit_mode: Rc::new(EditMode::None),
                        edit_failed: false,
                        right_panel_text_field: Rc::new(String::default()),
                        options: Rc::new(screen_options),
                    });
                }
            }
            return Err("Failed to read config file.")?;
        }
        return Err("Home directory not set, please make a home directory.")?;
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
                    match (*self.edit_mode).clone() {
                        EditMode::None => match key.code {
                            KeyCode::Char('e') => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if self.local_menu_state.borrow().selected().unwrap()
                                        == self.local_decks_names.len() - 1
                                    {
                                        continue;
                                    }
                                    let mut state = ListState::default();
                                    state.select(Some(0));
                                    self.edit_mode =
                                        Rc::new(EditMode::EditMenu(Rc::new(RefCell::new(state))));
                                }
                                ScreenState::DeckEditor => {
                                    if self.edit_menu_state.borrow().selected().unwrap()
                                        == self.current_deck.borrow().len()
                                    {
                                        continue;
                                    }
                                    self.edit_mode = Rc::new(EditMode::EditContent);
                                    self.current_deck.borrow_mut().cur_card =
                                        self.edit_menu_state.borrow().selected().unwrap();
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    if self.current_deck.borrow().contents[current_card].len()
                                        == current_section
                                    {
                                        self.right_panel_text_field = Rc::new(String::default());
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections
                                            .push(String::default());
                                    } else {
                                        self.right_panel_text_field = Rc::new(
                                            self.current_deck.borrow().contents[current_card]
                                                .sections[current_section]
                                                .clone(),
                                        );
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Char('d') => match *initial_state {
                                ScreenState::LocalMenu => {
                                    let current_deck =
                                        self.local_menu_state.borrow().selected().unwrap();
                                    if current_deck == self.local_decks_names.len() - 1 {
                                        continue;
                                    }
                                    let mut temp_vec = self.local_decks_names.clone().to_vec();
                                    let mut deck_path = self.options.local_directory.clone();
                                    deck_path.push(&temp_vec[current_deck]);
                                    fs::remove_dir_all(deck_path)?;
                                    temp_vec.remove(current_deck);
                                    self.local_decks_names = temp_vec.into_boxed_slice();
                                    let end = self.local_decks_names.len() - 1;
                                    if current_deck >= end {
                                        self.local_menu_state.borrow_mut().select(Some(end - 1));
                                    }
                                }
                                ScreenState::DeckEditor => {
                                    let current_card =
                                        self.edit_menu_state.borrow().selected().unwrap();
                                    if current_card == self.current_deck.borrow().len() {
                                        continue;
                                    }
                                    let mut temp_vec =
                                        self.current_deck.borrow().contents.clone().into_vec();
                                    let mut card_path = self.options.local_directory.clone();
                                    card_path.push(&self.current_deck.borrow().deck_title);
                                    card_path.push(temp_vec[current_card].saved_name());
                                    fs::remove_file(card_path)?;
                                    temp_vec.remove(current_card);
                                    self.current_deck.borrow_mut().contents =
                                        temp_vec.into_boxed_slice();
                                    let end = self.current_deck.borrow().len();
                                    if current_card >= end {
                                        self.edit_menu_state.borrow_mut().select(Some(end - 1));
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Char('q') => match *initial_state {
                                ScreenState::DeckEditor => {
                                    self.current_deck
                                        .take()
                                        .write_to_dir(self.options.local_directory.clone())?;
                                    break;
                                }
                                _ => break,
                            },
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
                                ScreenState::DeckEditor => {
                                    let new_state = util::offset_state(
                                        &self.edit_menu_state.clone().borrow(),
                                        1,
                                        true,
                                        self.current_deck.borrow().len(),
                                    );
                                    self.edit_menu_state = Rc::new(RefCell::new(new_state));
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
                                ScreenState::DeckEditor => {
                                    let new_state = util::offset_state(
                                        &self.edit_menu_state.clone().borrow(),
                                        1,
                                        false,
                                        self.current_deck.borrow().len(),
                                    );
                                    self.edit_menu_state = Rc::new(RefCell::new(new_state));
                                }
                                _ => (),
                            },
                            KeyCode::Enter => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if self
                                        .local_menu_state
                                        .borrow()
                                        .selected()
                                        .unwrap_or_default()
                                        == self.local_decks_names.len() - 1
                                    {
                                        self.edit_mode = Rc::new(EditMode::AddItem);
                                    } else {
                                        let mut cur_dir: PathBuf =
                                            self.options.local_directory.clone();
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
                                }
                                ScreenState::DeckEditor => {
                                    if self.edit_menu_state.borrow().selected().unwrap_or_default()
                                        == self.current_deck.borrow().len()
                                    {
                                        self.edit_mode = Rc::new(EditMode::AddItem);
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Right => match *initial_state {
                                ScreenState::DeckViewer => {
                                    (*self.current_deck).borrow_mut().increment_deck(true);
                                }
                                _ => (),
                            },
                            KeyCode::Left => match *initial_state {
                                ScreenState::DeckViewer => {
                                    (*self.current_deck).borrow_mut().decrement_deck(true);
                                }
                                _ => (),
                            },
                            KeyCode::Esc => match *initial_state {
                                ScreenState::DeckViewer => {
                                    self.current_deck = Rc::new(RefCell::new(Deck::default()));
                                    self.state = Rc::new(ScreenState::LocalMenu);
                                }
                                ScreenState::DeckEditor => {
                                    self.state = Rc::new(ScreenState::LocalMenu);
                                    let mut state = ListState::default();
                                    state.select(Some(0));
                                    self.edit_mode =
                                        Rc::new(EditMode::EditMenu(Rc::new(RefCell::new(state))));
                                    self.current_deck
                                        .take()
                                        .write_to_dir(self.options.local_directory.clone())?;
                                    self.current_deck = Rc::new(RefCell::new(Deck::default()));
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        EditMode::EditMenu(menu_state) => match key.code {
                            KeyCode::Up => match *initial_state {
                                ScreenState::LocalMenu => {
                                    let new_state =
                                        util::offset_state(&menu_state.borrow(), 1, false, 1);
                                    self.edit_mode = Rc::new(EditMode::EditMenu(Rc::new(
                                        RefCell::new(new_state),
                                    )));
                                }
                                _ => (),
                            },
                            KeyCode::Down => match *initial_state {
                                ScreenState::LocalMenu => {
                                    let new_state =
                                        util::offset_state(&menu_state.borrow(), 1, true, 1);
                                    self.edit_mode = Rc::new(EditMode::EditMenu(Rc::new(
                                        RefCell::new(new_state),
                                    )));
                                }
                                _ => (),
                            },

                            KeyCode::Esc => match *initial_state {
                                ScreenState::LocalMenu => {
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                _ => (),
                            },
                            KeyCode::Enter => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if let Some(item_index) = menu_state.borrow().selected() {
                                        if item_index == 0 {
                                            if let Some(current_selection) =
                                                self.local_menu_state.borrow().selected()
                                            {
                                                self.right_panel_text_field = Rc::new(
                                                    self.local_decks_names[current_selection]
                                                        .clone(),
                                                );
                                            }
                                            self.edit_mode = Rc::new(EditMode::EditContent);
                                        } else {
                                            let mut cur_dir: PathBuf =
                                                self.options.local_directory.clone();
                                            cur_dir.push(
                                                &self.local_decks_names[self
                                                    .local_menu_state
                                                    .borrow()
                                                    .selected()
                                                    .unwrap_or_default()],
                                            );
                                            if let Ok(deck) = Deck::read_from_dir(&cur_dir) {
                                                self.current_deck = Rc::new(RefCell::new(deck));
                                                self.state = Rc::new(ScreenState::DeckEditor);
                                                self.edit_mode = Rc::new(EditMode::None);
                                            }
                                        }
                                    }
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        EditMode::AddItem => match key.code {
                            KeyCode::Char(typed_char) => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if self.edit_failed {
                                        self.edit_failed = false;
                                    }
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.push(typed_char);
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                ScreenState::DeckEditor => {
                                    if self.edit_failed {
                                        self.edit_failed = false;
                                    }
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.push(typed_char);
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                _ => (),
                            },
                            KeyCode::Enter => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if !self.right_panel_text_field.is_empty() {
                                        let mut new_dir = self.options.local_directory.clone();
                                        new_dir.push(self.right_panel_text_field.to_string());
                                        if let Ok(_) = fs::create_dir(new_dir) {
                                            let mut temp_vec = self.local_decks_names.to_vec();
                                            temp_vec.insert(
                                                temp_vec.len() - 1,
                                                self.right_panel_text_field.to_string(),
                                            );
                                            self.local_decks_names = temp_vec.into_boxed_slice();
                                            self.right_panel_text_field =
                                                Rc::new(String::default());
                                            self.edit_mode = Rc::new(EditMode::None);
                                            terminal.clear()?;
                                        } else {
                                            self.edit_failed = true;
                                        }
                                    } else {
                                        self.right_panel_text_field = Rc::new(String::default());
                                        self.edit_mode = Rc::new(EditMode::None);
                                        terminal.clear()?;
                                    }
                                }
                                ScreenState::DeckEditor => {
                                    if !self.right_panel_text_field.is_empty() {
                                        if !self
                                            .current_deck
                                            .borrow()
                                            .get_card_names()
                                            .contains(&self.right_panel_text_field)
                                        {
                                            let mut temp_vec =
                                                self.current_deck.borrow_mut().contents.to_vec();
                                            temp_vec.push(Card::new(
                                                self.right_panel_text_field.clone().to_string(),
                                            ));
                                            self.current_deck.borrow_mut().contents =
                                                temp_vec.into_boxed_slice();
                                            self.current_deck.take().write_to_dir(
                                                self.options.local_directory.clone(),
                                            )?;
                                            let mut cur_dir: PathBuf =
                                                self.options.local_directory.clone();
                                            cur_dir.push(
                                                &self.local_decks_names[self
                                                    .local_menu_state
                                                    .borrow()
                                                    .selected()
                                                    .unwrap_or_default()],
                                            );
                                            if let Ok(deck) = Deck::read_from_dir(&cur_dir) {
                                                self.current_deck = Rc::new(RefCell::new(deck));
                                            }

                                            self.right_panel_text_field =
                                                Rc::new(String::default());
                                            self.edit_mode = Rc::new(EditMode::None);
                                            terminal.clear()?;
                                        } else {
                                            self.edit_failed = true;
                                        }
                                    } else {
                                        self.right_panel_text_field = Rc::new(String::default());
                                        self.edit_mode = Rc::new(EditMode::None);
                                        terminal.clear()?;
                                    }
                                }
                                _ => (),
                            },
                            KeyCode::Esc => match *initial_state {
                                ScreenState::LocalMenu => {
                                    self.right_panel_text_field = Rc::new(String::default());
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                ScreenState::DeckEditor => {
                                    self.right_panel_text_field = Rc::new(String::default());
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                _ => (),
                            },
                            KeyCode::Backspace => match *initial_state {
                                ScreenState::LocalMenu => {
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.pop();
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                ScreenState::DeckEditor => {
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.pop();
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        EditMode::EditContent => match (key.code, key.modifiers) {
                            (KeyCode::Char('a'), KeyModifiers::CONTROL) => match *initial_state {
                                ScreenState::DeckEditor => {
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    self.current_deck.borrow_mut().contents[current_card]
                                        .sections[current_section] =
                                        self.right_panel_text_field.clone().to_string();
                                    self.current_deck.borrow_mut().contents[current_card]
                                        .sections
                                        .insert(current_section, String::from(""));
                                    self.right_panel_text_field = Rc::new(String::default());
                                }
                                _ => (),
                            },
                            (KeyCode::Char('d'), KeyModifiers::CONTROL) => match *initial_state {
                                ScreenState::DeckEditor => {
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    if !self.current_deck.borrow().contents[current_card]
                                        .sections
                                        .is_empty()
                                    {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections
                                            .remove(current_section);
                                        let sections_vec_length =
                                            self.current_deck.borrow().contents[current_card].len();
                                        if sections_vec_length == current_section {
                                            if sections_vec_length == 0 {
                                                self.right_panel_text_field =
                                                    Rc::new(String::default());
                                            } else {
                                                self.right_panel_text_field = Rc::new(
                                                    self.current_deck.borrow().contents
                                                        [current_card]
                                                        .sections[current_section - 1]
                                                        .clone(),
                                                );
                                                self.current_deck.borrow_mut().contents
                                                    [current_card]
                                                    .decrement_section();
                                            }
                                        } else {
                                            self.right_panel_text_field = Rc::new(
                                                self.current_deck.borrow().contents[current_card]
                                                    .sections[current_section]
                                                    .clone(),
                                            );
                                        }
                                    }
                                }
                                _ => (),
                            },
                            (
                                KeyCode::Char(typed_char),
                                KeyModifiers::NONE | KeyModifiers::SHIFT,
                            ) => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if self.edit_failed {
                                        self.edit_failed = false;
                                    }
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.push(typed_char);
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                ScreenState::DeckEditor => {
                                    if self.edit_failed {
                                        self.edit_failed = false;
                                    }
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.push(typed_char);
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                _ => (),
                            },
                            (KeyCode::Enter, KeyModifiers::NONE) => match *initial_state {
                                ScreenState::LocalMenu => {
                                    if !self.right_panel_text_field.is_empty() {
                                        if self
                                            .local_decks_names
                                            .contains(&self.right_panel_text_field)
                                        {
                                            self.edit_failed = true;
                                            continue;
                                        }
                                        let mut old_dir = self.options.local_directory.clone();
                                        old_dir.push(
                                            self.local_decks_names[self
                                                .local_menu_state
                                                .borrow()
                                                .selected()
                                                .unwrap()]
                                            .clone(),
                                        );
                                        let mut new_dir = self.options.local_directory.clone();
                                        new_dir.push(self.right_panel_text_field.to_string());
                                        if let Ok(_) = fs::rename(old_dir, new_dir) {
                                            let mut temp_vec = self.local_decks_names.to_vec();
                                            temp_vec.push(self.right_panel_text_field.to_string());
                                            temp_vec.swap_remove(
                                                self.local_menu_state.borrow().selected().unwrap(),
                                            );
                                            self.local_decks_names = temp_vec.into_boxed_slice();
                                            self.right_panel_text_field =
                                                Rc::new(String::default());
                                            self.edit_mode = Rc::new(EditMode::None);
                                            terminal.clear()?;
                                        } else {
                                            self.edit_failed = true;
                                        }
                                    } else {
                                        self.right_panel_text_field = Rc::new(String::default());
                                        self.edit_mode = Rc::new(EditMode::None);
                                        terminal.clear()?;
                                    }
                                }
                                ScreenState::DeckEditor => {
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    if self.current_deck.borrow().contents[current_card].len() == 0
                                    {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections
                                            .push(self.right_panel_text_field.clone().to_string());
                                    } else {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections[current_section] =
                                            self.right_panel_text_field.clone().to_string();
                                    }
                                    self.right_panel_text_field = Rc::new(String::default());
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                _ => (),
                            },
                            (KeyCode::Esc, KeyModifiers::NONE) => match *initial_state {
                                ScreenState::LocalMenu => {
                                    self.right_panel_text_field = Rc::new(String::default());
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                ScreenState::DeckEditor => {
                                    self.right_panel_text_field = Rc::new(String::default());
                                    self.edit_mode = Rc::new(EditMode::None);
                                    terminal.clear()?;
                                }
                                _ => (),
                            },
                            (KeyCode::Backspace, KeyModifiers::NONE) => match *initial_state {
                                ScreenState::LocalMenu => {
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.pop();
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                ScreenState::DeckEditor => {
                                    let mut current_name = (*self.right_panel_text_field).clone();
                                    current_name.pop();
                                    self.right_panel_text_field = Rc::new(current_name);
                                }
                                _ => (),
                            },
                            (KeyCode::Right, KeyModifiers::NONE) => match *initial_state {
                                ScreenState::DeckEditor => {
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    if self.current_deck.borrow().contents[current_card].len() == 0
                                    {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections
                                            .push(self.right_panel_text_field.clone().to_string());
                                    } else {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections[current_section] =
                                            self.right_panel_text_field.clone().to_string();
                                    }
                                    (*self.current_deck).borrow_mut().increment_deck(false);
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    self.right_panel_text_field = Rc::new(
                                        self.current_deck.borrow().contents[current_card].sections
                                            [current_section]
                                            .clone(),
                                    );
                                }
                                _ => (),
                            },
                            (KeyCode::Left, KeyModifiers::NONE) => match *initial_state {
                                ScreenState::DeckEditor => {
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    if self.current_deck.borrow().contents[current_card].len() == 0
                                    {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections
                                            .push(self.right_panel_text_field.clone().to_string());
                                    } else {
                                        self.current_deck.borrow_mut().contents[current_card]
                                            .sections[current_section] =
                                            self.right_panel_text_field.clone().to_string();
                                    }
                                    (*self.current_deck).borrow_mut().decrement_deck(false);
                                    let current_card = self.current_deck.borrow().cur_card;
                                    let current_section = self.current_deck.borrow().contents
                                        [current_card]
                                        .current_section;
                                    self.right_panel_text_field = Rc::new(
                                        self.current_deck.borrow().contents[current_card].sections
                                            [current_section]
                                            .clone(),
                                    );
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                    }
                }
            }
            //First we find the areas of the screen we are drawing to, then we draw each part of the screen using the appropriate function.
            let menu_layout = Screen::build_layout(&mut terminal.get_frame());
            terminal.draw(|f| {
                self.render_header(f, &menu_layout[0]);
                self.render_footer(f, &menu_layout[4]);
                self.render_middle_panel_content(f, &menu_layout[2]);
                self.render_right_panel_content(f, &menu_layout[3]);
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
            ScreenState::DeckEditor => {
                let titles = vec![Spans::from(vec![
                    Span::raw("Deck: "),
                    Span::raw(self.current_deck.borrow().deck_title.clone()),
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
        };
    }

    fn render_footer(&self, f: &mut tui::Frame<CrosstermBackend<Stdout>>, area: &Rect) -> () {
        match *self.state.clone() {
            ScreenState::LocalMenu => match *self.edit_mode {
                EditMode::None => {
                    let mut text_vec = vec![
                        Span::raw("Selected '"),
                        Span::raw(
                            self.local_decks_names
                                [self.local_menu_state.borrow().selected().unwrap()]
                            .clone(),
                        ),
                        Span::raw("' Navigate (↑/↓) Select (Enter) "),
                        Span::raw("(e)dit "),
                        Span::raw("(d)elete "),
                        Span::raw("(q)uit"),
                    ];
                    if self.local_menu_state.borrow().selected().unwrap()
                        == (self.local_decks_names.len() - 1)
                    {
                        let keep = [true, true, true, false, false, true];
                        let mut iter = keep.iter();
                        text_vec.retain(|_| *iter.next().unwrap());
                    }
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
                EditMode::EditMenu(_) => {
                    let text_vec = vec![Span::raw("Navigate (↑/↓) Select (Enter) Go Back (esc)")];
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
                EditMode::AddItem => {
                    let text_vec = vec![Span::raw("Save (Enter) Go back (esc) Undo (Backspace)")];
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
                EditMode::EditContent => {
                    let text_vec = vec![Span::raw("Save (Enter) Go back (esc) Undo (Backspace)")];
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
            },
            ScreenState::DeckViewer => {
                let text = vec![Spans::from(vec![Span::raw(
                    "Next Section/Next Card (←/→) Return to Menu (Esc) (q)uit",
                )])];
                let footer = Paragraph::new(text)
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                    .alignment(Alignment::Left);
                f.render_widget(footer, *area);
            }
            ScreenState::DeckEditor => match *self.edit_mode {
                EditMode::None => {
                    let mut temp_vec = self.current_deck.borrow().get_card_names();
                    temp_vec.push(String::from("Add new card..."));
                    let mut text_vec = vec![
                        Span::raw("Selected '"),
                        Span::raw(
                            temp_vec[self.edit_menu_state.borrow().selected().unwrap()].clone(),
                        ),
                        Span::raw("' Navigate (↑/↓) Select (Enter) "),
                        Span::raw("(e)dit "),
                        Span::raw("(d)elete "),
                        Span::raw("(q)uit"),
                    ];
                    if self.local_menu_state.borrow().selected().unwrap()
                        == (self.local_decks_names.len() - 1)
                    {
                        let keep = [true, true, true, false, false, true];
                        let mut iter = keep.iter();
                        text_vec.retain(|_| *iter.next().unwrap());
                    }
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
                EditMode::EditContent => {
                    let mut temp_vec = self.current_deck.borrow().get_card_names();
                    temp_vec.push(String::from("Add new card..."));
                    let text_vec = vec![Span::raw(
                        "Next Section (←/→) Finish (Enter) Delete (ctrl-d) Add (ctrl-a) (q)uit",
                    )];
                    let text = vec![Spans::from(text_vec)];
                    let footer = Paragraph::new(text)
                        .block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
                        .alignment(Alignment::Left);
                    f.render_widget(footer, *area);
                }
                _ => (),
            },
        }
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
                f.render_widget(self.current_deck.borrow().as_widget(), *area)
            }
            ScreenState::DeckEditor => {
                let mut name_vec = self.current_deck.borrow().get_card_names();
                name_vec.push(String::from("Add new card..."));
                let list_items: Vec<ListItem> = name_vec
                    .iter()
                    .map(|name| ListItem::new(name.to_owned()))
                    .collect();
                let middle_panel = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
                //The state needs this weird configuration to work, sadly just how it is.
                f.render_stateful_widget(
                    middle_panel,
                    *area,
                    &mut (*self.edit_menu_state).borrow_mut(),
                );
            }
        }
    }

    fn render_right_panel_content(
        &self,
        f: &mut tui::Frame<CrosstermBackend<Stdout>>,
        area: &Rect,
    ) -> () {
        match *self.state.clone() {
            ScreenState::LocalMenu => match &*self.edit_mode {
                EditMode::EditMenu(menu_state) => {
                    let list_items = vec![
                        ListItem::new("Edit Deck Name"),
                        ListItem::new("Edit/Add Cards"),
                    ];
                    let right_panel = List::new(list_items)
                        .block(Block::default().borders(Borders::ALL).title(" Edit Menu "))
                        .style(Style::default().fg(Color::White))
                        .highlight_style(Style::default().bg(Color::White).fg(Color::Black));
                    f.render_stateful_widget(right_panel, *area, &mut menu_state.borrow_mut())
                }
                EditMode::AddItem => {
                    let text = vec![Spans::from((*self.right_panel_text_field).clone())];

                    let right_panel = Paragraph::new(text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" New Deck Name "),
                    );
                    let right_panel_layout = Layout::default()
                        .constraints(
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(20),
                                Constraint::Percentage(60),
                            ]
                            .as_ref(),
                        )
                        .split(*area);
                    f.render_widget(right_panel, right_panel_layout[0]);
                    if self.edit_failed {
                        let error_text = vec![Spans::from("Deck already exists.")];
                        let right_panel_error = Paragraph::new(error_text)
                            .block(Block::default().borders(Borders::ALL).title(" Error "));
                        f.render_widget(right_panel_error, right_panel_layout[1]);
                    }
                }
                EditMode::EditContent => {
                    let text = vec![Spans::from((*self.right_panel_text_field).clone())];

                    let right_panel = Paragraph::new(text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Change Deck Name "),
                    );
                    let right_panel_layout = Layout::default()
                        .constraints(
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(20),
                                Constraint::Percentage(60),
                            ]
                            .as_ref(),
                        )
                        .split(*area);
                    f.render_widget(right_panel, right_panel_layout[0]);
                    if self.edit_failed {
                        let error_text = vec![Spans::from("Deck already exists.")];
                        let right_panel_error = Paragraph::new(error_text)
                            .block(Block::default().borders(Borders::ALL).title(" Error "));
                        f.render_widget(right_panel_error, right_panel_layout[1]);
                    }
                }
                _ => (),
            },
            ScreenState::DeckEditor => match &*self.edit_mode {
                EditMode::AddItem => {
                    let text = vec![Spans::from((*self.right_panel_text_field).clone())];

                    let right_panel = Paragraph::new(text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" New Card Name "),
                    );
                    let right_panel_layout = Layout::default()
                        .constraints(
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(20),
                                Constraint::Percentage(60),
                            ]
                            .as_ref(),
                        )
                        .split(*area);
                    f.render_widget(right_panel, right_panel_layout[0]);
                    if self.edit_failed {
                        let error_text = vec![Spans::from("Card already exists.")];
                        let right_panel_error = Paragraph::new(error_text)
                            .block(Block::default().borders(Borders::ALL).title(" Error "));
                        f.render_widget(right_panel_error, right_panel_layout[1]);
                    }
                }
                EditMode::EditContent => {
                    let text = vec![Spans::from((*self.right_panel_text_field).clone())];

                    let right_panel = Paragraph::new(text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Change Card Section "),
                    );
                    let right_panel_layout = Layout::default()
                        .constraints(
                            [Constraint::Percentage(80), Constraint::Percentage(20)].as_ref(),
                        )
                        .split(*area);
                    f.render_widget(right_panel, right_panel_layout[0]);
                    if self.edit_failed {
                        let error_text = vec![Spans::from("How did you get here?")];
                        let right_panel_error = Paragraph::new(error_text)
                            .block(Block::default().borders(Borders::ALL).title(" Error "));
                        f.render_widget(right_panel_error, right_panel_layout[1]);
                    }
                }
                _ => (),
            },
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

    fn get_current_local_decks(options: &ScreenOptions) -> Vec<String> {
        let cur_dir = options.local_directory.clone();
        let mut list_items_str =
            util::get_sub_directories(cur_dir.as_path()).expect("Failed to read directories.");
        list_items_str.sort();
        list_items_str.push(String::from("Add new deck..."));
        list_items_str
    }
}
