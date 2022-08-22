use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Card {
    title: String,
    sections: Vec<String>,
}

impl Card {
    pub fn new() -> Self {
        return Card {
            title: String::from(""),
            sections: Vec::<String>::new(),
        };
    }

    pub fn read_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let input_text = fs::read_to_string(filename)?;
        let input_object: Result<Card, Box<dyn Error>> = serde_json::from_str(&input_text)
            .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));
        return input_object;
    }

    pub fn write_to_file(self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let object_string_result = serde_json::to_string(&self)
            .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));
        match object_string_result {
            Ok(object_string) => {
                return fs::write(filename, object_string)
                    .or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>))
            }
            Err(err) => return Err(err),
        }
    }

    pub fn set_title(self, title: &str) -> Self {
        return Card {
            title: title.to_owned(),
            sections: self.sections,
        };
    }

    pub fn set_sections(self, sections: Vec<String>) -> Self {
        return Card {
            title: self.title,
            sections: sections,
        };
    }

    //TODO: Add formatting to printing using a table library (add current section to enum?)
    //TODO: Add section incrementing and decrementing
}
