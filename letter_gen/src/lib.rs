use std::{
    ops::{Deref, DerefMut},
    task::Poll,
};

use tokio_stream::Stream;

static LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

#[derive(Default)]
pub struct LetterGenerator {
    current: usize,
    current_seq: LetterSequence,
}

impl LetterGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct LetterSequence(pub(crate) String);

impl Default for LetterSequence {
    fn default() -> Self {
        Self(String::from("A"))
    }
}

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

impl std::fmt::Display for LetterSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Stream for LetterGenerator {
    type Item = LetterSequence;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let current = self.current;
        let next_letter = current + 1;
        let current_seq = &*self.current_seq;
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
        Poll::Ready(Some(new_seq))
    }
}

#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;

    use crate::{LetterGenerator, LetterSequence};

    #[tokio::test]
    async fn first_letter_is_a() {
        let mut gen = LetterGenerator::new();
        let letter = gen.next().await;
        assert_eq!(letter, Some(LetterSequence("A".to_string())));
    }

    #[tokio::test]
    async fn letter_after_last_is_a() {
        let gen = LetterGenerator::new();
        let letter = gen.skip(25).next().await;
        assert_eq!(letter, Some(LetterSequence("ZA".to_string())));
    }
}
