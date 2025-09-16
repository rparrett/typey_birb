use bevy::{
    input::keyboard::{Key, KeyboardInput},
    platform::collections::HashSet,
    prelude::*,
};
use rand::{prelude::*, rng};

pub struct TypingPlugin;

#[derive(Resource)]
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
        words.shuffle(&mut rng());
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
            self.words.shuffle(&mut rng());
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
        app.init_resource::<WordList>()
            .add_systems(Update, new_words)
            .add_systems(Update, keyboard);
    }
}

fn new_words(
    mut events: MessageReader<crate::Action>,
    mut query: Query<(Entity, &mut TypingTarget)>,
    mut wordlist: ResMut<WordList>,
) {
    for e in events.read() {
        if let crate::Action::NewWord(entity) = e {
            // build a list of characters to avoid for the next word,
            // skipping the word we're replacing.
            let not: HashSet<char> = query
                .iter()
                .filter(|(e, _)| e != entity)
                .flat_map(|(_, t)| t.word.chars())
                .collect();

            if let Ok((_, mut target)) = query.get_mut(*entity) {
                let next = wordlist.find_next_word(&not);
                target.replace(next);
            }
        }
    }
}

fn keyboard(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut query: Query<(Entity, &mut TypingTarget)>,
    mut events: MessageWriter<crate::Action>,
) {
    for event in keyboard_events.read() {
        let mut ok = false;

        if !event.state.is_pressed() {
            continue;
        };

        let Key::Character(ref key_str) = event.logical_key else {
            continue;
        };

        let Some(char) = key_str.chars().last() else {
            continue;
        };

        for (entity, mut target) in query.iter_mut() {
            let Some(next) = target.current_char() else {
                continue;
            };

            if next != char {
                continue;
            }

            for action in target.letter_actions.iter() {
                events.write(action.clone());
            }

            if target.advance_char().is_none() {
                events.write(crate::Action::NewWord(entity));

                for action in target.word_actions.iter() {
                    events.write(action.clone());
                }
            }

            ok = true;
        }

        if !ok {
            events.write(crate::Action::BadFlap);
        }
    }
}
