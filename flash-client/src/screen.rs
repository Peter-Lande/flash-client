use crate::util;
#[derive(Clone)]
pub enum ScreenState {
    //The field represents the state of the deck cursor
    LocalMenu(tui::widgets::ListState, Box<[String]>),
}

pub struct Screen {
    terminal: tui::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>>,
    state: ScreenState,
}

impl Screen {
    pub fn new(state: ScreenState) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_term = tui::Terminal::new(tui::backend::CrosstermBackend::new(std::io::stdout()))?;
        return Ok(Screen {
            terminal: temp_term,
            state: state,
        });
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            self.terminal_mut().backend_mut(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        match &self.state {
            ScreenState::LocalMenu(list_state, _) => {
                self.state = ScreenState::LocalMenu(
                    list_state.to_owned(),
                    Screen::get_current_decks().into_boxed_slice(),
                )
            }
        }
        return Ok(());
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // match &self.state {
        //     ScreenState::LocalMenu(list_state, decks) => {
        //         let mut new_state = list_state.clone();
        //         new_state.select(Some(1));
        //         self.state = ScreenState::LocalMenu(new_state, decks.to_owned());
        //     }
        // }
        match self.state.clone() {
            ScreenState::LocalMenu(mut list_state, current_decks) => {
                self.terminal_mut().draw(|f| {
                    let container = Screen::build_layout(f);
                    let header = Screen::build_header(&ScreenState::LocalMenu(
                        list_state.clone(),
                        Vec::new().into_boxed_slice(),
                    ));
                    f.render_widget(header, container[0]);
                    let middle_panel_content = Screen::build_main_panel_content(
                        &ScreenState::LocalMenu(list_state.clone(), current_decks),
                    );
                    f.render_stateful_widget(middle_panel_content, container[2], &mut list_state);
                })?;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10000));
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal_mut().backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        self.terminal_mut().show_cursor()?;
        Ok(())
    }

    fn terminal_mut(
        &mut self,
    ) -> &mut tui::terminal::Terminal<tui::backend::CrosstermBackend<std::io::Stdout>> {
        return &mut self.terminal;
    }

    fn build_header(state: &ScreenState) -> tui::widgets::Tabs<'static> {
        match state {
            ScreenState::LocalMenu(..) => {
                let titles = vec![
                    tui::text::Spans::from(tui::text::Span::raw("Local")),
                    tui::text::Spans::from(tui::text::Span::raw("Remote")),
                    tui::text::Spans::from(tui::text::Span::raw("Exit")),
                ];
                tui::widgets::Tabs::new(titles)
                    .block(
                        tui::widgets::Block::default()
                            .title(" Flash ")
                            .borders(tui::widgets::Borders::TOP | tui::widgets::Borders::BOTTOM)
                            .title_alignment(tui::layout::Alignment::Center),
                    )
                    .style(tui::style::Style::default().fg(tui::style::Color::White))
                    .highlight_style(
                        tui::style::Style::default()
                            .fg(tui::style::Color::Green)
                            .add_modifier(tui::style::Modifier::BOLD),
                    )
                    .select(0)
            }
        }
    }

    fn build_layout<B: tui::backend::Backend>(f: &mut tui::Frame<B>) -> Vec<tui::layout::Rect> {
        let first_layer_layout = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .vertical_margin(1)
            .horizontal_margin(2)
            .constraints(
                [
                    tui::layout::Constraint::Percentage(20),
                    tui::layout::Constraint::Percentage(80),
                ]
                .as_ref(),
            )
            .split(f.size());
        let header_layer = first_layer_layout[0];
        let content_layer = first_layer_layout[1];
        let second_layer_layout = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints(
                [
                    tui::layout::Constraint::Percentage(25),
                    tui::layout::Constraint::Percentage(50),
                    tui::layout::Constraint::Percentage(25),
                ]
                .as_ref(),
            )
            .split(content_layer);
        return vec![
            header_layer,
            second_layer_layout[0],
            second_layer_layout[1],
            second_layer_layout[2],
        ];
    }

    fn build_main_panel_content(state: &ScreenState) -> tui::widgets::List<'static> {
        match state {
            ScreenState::LocalMenu(_, current_decks) => {
                let list_items: Vec<tui::widgets::ListItem> = current_decks
                    .iter()
                    .map(|x| tui::widgets::ListItem::new(x.to_owned()))
                    .collect();
                return tui::widgets::List::new(list_items)
                    .block(tui::widgets::Block::default().borders(tui::widgets::Borders::ALL))
                    .style(tui::style::Style::default().fg(tui::style::Color::White))
                    .highlight_style(
                        tui::style::Style::default()
                            .bg(tui::style::Color::White)
                            .fg(tui::style::Color::Black),
                    );
            }
        }
    }
    fn get_current_decks() -> Vec<String> {
        let mut cur_dir = std::env::current_exe().expect("Could not find path to executable.");
        cur_dir.pop();
        cur_dir.push("decks");
        cur_dir.push("local/");
        let mut list_items_str =
            util::get_sub_directories(cur_dir.as_path()).expect("Failed to read directories.");
        list_items_str.sort();
        list_items_str
    }
}
