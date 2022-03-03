use bevy::{prelude::*, utils::HashSet};
use rand::prelude::*;

pub struct TypingPlugin;

pub struct WordList {
    words: Vec<String>,
    index: usize,
}

impl Default for WordList {
    fn default() -> Self {
        let mut words = crate::words::WORDS
            .lines()
            .map(|w| w.to_owned())
            .filter(|w| w.chars().count() > 0)
            .collect::<Vec<_>>();
        words.shuffle(&mut thread_rng());
        Self { words, index: 0 }
    }
}

impl WordList {
    pub fn find_next_word(&mut self, not: &HashSet<char>) -> String {
        loop {
            let next = self.advance_word();
            if next.chars().all(|c| !not.contains(&c)) {
                return next;
            }
        }
    }

    fn advance_word(&mut self) -> String {
        self.index += 1;
        if self.index >= self.words.len() {
            self.words.shuffle(&mut thread_rng());
            self.index = 0;
        }
        self.words[self.index].clone()
    }
}

#[derive(Component)]
pub struct TypingTarget {
    pub letter_actions: Vec<crate::Action>,
    pub word_actions: Vec<crate::Action>,
    pub index: usize,
    pub word: String,
}

impl TypingTarget {
    pub fn new(word: String, actions: Vec<crate::Action>) -> Self {
        Self {
            letter_actions: actions,
            word_actions: vec![],
            index: 0,
            word,
        }
    }
    pub fn new_whole(word: String, actions: Vec<crate::Action>) -> Self {
        Self {
            word_actions: actions,
            letter_actions: vec![],
            index: 0,
            word,
        }
    }
    pub fn current_char(&self) -> Option<char> {
        self.word.chars().nth(self.index)
    }
    pub fn advance_char(&mut self) -> Option<char> {
        self.index += 1;
        self.current_char()
    }
    pub fn replace(&mut self, new: String) {
        self.word = new;
        self.index = 0;
    }
}

impl Plugin for TypingPlugin {
    fn build(&self, app: &mut App) {
        // We need the font to have been loaded for this to work.
        app.init_resource::<WordList>()
            .add_system(new_words)
            .add_system(keyboard);
    }
}

fn new_words(
    mut events: EventReader<crate::Action>,
    mut query: Query<&mut TypingTarget>,
    mut wordlist: ResMut<WordList>,
) {
    for e in events.iter() {
        match e {
            crate::Action::NewWord(entity) => {
                let not: HashSet<char> = query.iter().map(|t| t.word.chars()).flatten().collect();

                if let Ok(mut target) = query.get_mut(*entity) {
                    let next = wordlist.find_next_word(&not);
                    target.replace(next);
                }
            }
            _ => {}
        }
    }
}

fn keyboard(
    mut char_input_events: EventReader<ReceivedCharacter>,
    mut query: Query<(Entity, &mut TypingTarget)>,
    mut events: EventWriter<crate::Action>,
) {
    for event in char_input_events.iter() {
        for (entity, mut target) in query.iter_mut() {
            if let Some(next) = target.current_char() {
                if next == event.char {
                    for action in target.letter_actions.iter() {
                        events.send(action.clone());
                    }

                    if target.advance_char().is_none() {
                        events.send(crate::Action::NewWord(entity));

                        for action in target.word_actions.iter() {
                            events.send(action.clone());
                        }
                    }
                }
            }
        }
    }
}
