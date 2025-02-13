use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::Poll,
};

use serde::{Deserialize, Serialize};
use tokio_stream::Stream;

static LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

pub struct LetterGenerator {
    current: usize,
    current_seq: LetterSequence,
}

impl LetterGenerator {
    pub fn new() -> Self {
        Self {
            current: 0,
            current_seq: LetterSequence("A".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LetterSequence(pub String);

impl Deref for LetterSequence {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LetterSequence {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Stream for LetterGenerator {
    type Item = LetterSequence;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let next_letter = self.current + 1;
        let current_seq = &self.current_seq.0;
        let last_char = &current_seq[current_seq.len() - 1..];

        let mut s = String::from(current_seq);

        if last_char == LETTERS[LETTERS.len() - 1] {
            s.push_str(LETTERS[0]);

            self.current = 0;
        } else {
            s.pop();
            s.push_str(LETTERS[next_letter]);
            self.current = next_letter;
        }

        let new_seq = LetterSequence(s);
        self.current_seq = new_seq.clone();
        return Poll::Ready(Some(new_seq));
    }
}

pub enum Message {
    Progress(String),
}

impl Message {
    pub fn get_content(self) -> String {
        match self {
            Message::Progress(content) => content,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Playlist {
    pub number: u8,
    pub people: Vec<Person>,
    pub format: String,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            number: 0,
            format: String::from("mp4"),
            people: vec![
                Person {
                    index: 0,
                    link: String::new(),
                },
                Person {
                    index: 1,
                    link: String::new(),
                },
                Person {
                    index: 2,
                    link: String::new(),
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    pub index: usize,
    pub link: String,
}
