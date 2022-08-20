pub enum Card {
    Title(Vec<String>),
    Flash(Vec<String>),
    Error(String),
    None,
}

impl Card {
    pub fn new() -> Self {
        return Card::None;
    }
    pub fn make_title(self) -> Self {
        match self {
            Card::Title(info) => return Card::Title(info),
            Card::Flash(info) => return Card::Title(info),
            Card::Error(message) => return Card::Error(message),
            Card::None => return Card::Title(Vec::new()),
        }
    }
    pub fn make_flash(self) -> Self {
        match self {
            Card::Title(info) => return Card::Flash(info),
            Card::Flash(info) => return Card::Flash(info),
            Card::Error(message) => return Card::Error(message),
            Card::None => return Card::Title(Vec::new()),
        }
    }
    //TODO: Add ability to convert file to card and card to file

    //TODO: Add formatting to printing using a table library (add current section to enum?)
    //TODO: Add section incrementing and decrementing
}

impl From<Vec<String>> for Card {
    fn from(input: Vec<String>) -> Self {
        if input.len() != 0 {
            match input[0].as_str() {
                "TITLE" => return Card::Title(input[1..].to_vec()),
                "FLASH" => return Card::Flash(input[1..].to_vec()),
                _ => return Card::Error(String::from("Card type does not exist.")),
            }
        } else {
            return Card::None;
        }
    }
}

impl From<String> for Card {
    fn from(input: String) -> Self {
        return Card::from(
            input
                .split("...")
                .map(|string| string.to_owned())
                .collect::<Vec<String>>(),
        );
    }
}
