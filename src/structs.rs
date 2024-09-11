use serde::{Deserialize, Serialize};

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
