use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tui::layout::Alignment;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Widget};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Card {
    pub title: String,
    pub sections: Vec<String>,
    pub current_section: usize,
}

impl Card {
    pub fn new(title: String) -> Self {
        return Card {
            title: title,
            sections: Vec::new(),
            current_section: 0,
        };
    }
    pub fn read_from_file(filepath: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let input_text = fs::read_to_string(filepath)?;
        let input_object: Result<Card, Box<dyn Error>> = serde_json::from_str(&input_text)
            .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));
        return input_object;
    }

    pub fn write_to_file(self, mut parent_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        parent_path.push(&(self.title.clone() + ".json"));
        let file_path = parent_path;
        let object_string_result = serde_json::to_string(&self)
            .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));
        match object_string_result {
            Ok(object_string) => {
                return fs::write(file_path, object_string)
                    .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>))
            }
            Err(err) => return Err(err),
        }
    }

    pub fn increment_section(&mut self) -> Option<usize> {
        if let Some(i) = self.current_section.checked_add(1) {
            if i >= self.sections.len() {
                self.current_section = self.sections.len() - 1;
                return None;
            }
            self.current_section = i;
            return Some(i);
        } else {
            return None;
        }
    }

    pub fn decrement_section(&mut self) -> Option<usize> {
        if let Some(i) = self.current_section.checked_sub(1) {
            if i > self.sections.len() {
                self.current_section = self.sections.len();
                return None;
            }
            self.current_section = i;
            return Some(i);
        } else {
            return None;
        }
    }

    pub fn as_widget(&self) -> impl Widget {
        if !self.sections.is_empty() {
            let text = Spans::from(self.sections[self.current_section].to_owned());
            return Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.to_owned())
                    .title_alignment(Alignment::Center),
            );
        } else {
            let text = Spans::from("");
            return Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.to_owned())
                    .title_alignment(Alignment::Center),
            );
        }
    }

    pub fn len(&self) -> usize {
        return self.sections.len();
    }
}
