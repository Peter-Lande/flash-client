#[derive(Copy, Clone)]
pub enum ScreenState {
    //The u8 represents the state of the selection menu
    LocalMenu(u8),
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
        return Ok(());
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state;
        self.terminal_mut().draw(|f| {
            let container = Screen::build_layout(f);
            let header = Screen::build_header(state);
            f.render_widget(header, container[0]);
            let block = tui::widgets::Block::default().borders(tui::widgets::Borders::ALL);
            f.render_widget(block, container[2]);
        })?;
        std::thread::sleep(std::time::Duration::from_millis(5000));
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

    fn build_header(state: ScreenState) -> tui::widgets::Tabs<'static> {
        match state {
            ScreenState::LocalMenu(_) => {
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
            .vertical_margin(1)
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

    //fn build_main_panel_content(state: ScreenState) -> tui::widgets::List<'static> {}
}
