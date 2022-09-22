use std::{fs::read_dir, path::Path};

use crate::card::Card;

#[derive(Clone, Debug, Default)]
pub struct Deck {
    pub deck_title: String,
    contents: Box<[Card]>,
    pub cur_card: usize,
}

impl Deck {
    pub fn new(title: &str, cards: Vec<Card>) -> Self {
        return Deck {
            deck_title: title.to_string(),
            contents: cards.into_boxed_slice(),
            cur_card: 0,
        };
    }

    pub fn read_from_dir(dirpath: &Path) -> Result<Self, String> {
        let mut files: Vec<String> = Vec::new();
        if let Ok(entries) = read_dir(dirpath) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file()
                            && entry
                                .file_name()
                                .into_string()
                                .and_then(|x| Ok(x.contains(".json")))
                                .unwrap_or(false)
                        {
                            files.push(entry.file_name().into_string().unwrap());
                        }
                    }
                }
            }
            let cards: Vec<Card> = files
                .iter()
                .filter_map(|file_name| {
                    let card_option = match Card::read_from_file(&dirpath.join(file_name)) {
                        Ok(inner) => Some(inner),
                        Err(_) => None,
                    };
                    return card_option;
                })
                .collect();
            return Ok(Deck::new(
                dirpath
                    .file_stem()
                    .and_then(|dir_name| dir_name.to_str())
                    .unwrap_or("Unnamed"),
                cards,
            ));
        }
        return Err(String::from("Failed to read directory."));
    }

    pub fn increment_deck(&mut self) -> Option<usize> {
        if let None = self.contents[self.cur_card].increment_section() {
            if let Some(i) = self.cur_card.checked_add(1) {
                if self.cur_card > self.contents.len() {
                    self.cur_card = self.contents.len();
                    return Some(self.cur_card);
                } else {
                    return Some(i);
                }
            } else {
                return None;
            }
        } else {
            return Some(self.cur_card);
        }
    }

    pub fn decrement_deck(&mut self) -> Option<usize> {
        if let None = self.contents[self.cur_card].decrement_section() {
            if let Some(i) = self.cur_card.checked_sub(1) {
                if self.cur_card > self.contents.len() {
                    self.cur_card = self.contents.len();
                    return Some(self.cur_card);
                } else {
                    return Some(i);
                }
            } else {
                return None;
            }
        } else {
            return Some(self.cur_card);
        }
    }
    pub fn len(&self) -> usize {
        return self.contents.len();
    }
}
