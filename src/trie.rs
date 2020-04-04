struct LetterSet {
    letters: Vec<u8>,
    words: Vec<String>,
}

impl LetterSet {
    pub fn new(letters: Vec<u8>) -> Self {
        Self {
            letters,
            words: Vec::new(),
        }
    }
}

impl AsRef<[u8]> for LetterSet {
    fn as_ref(&self) -> &[u8] {
        self.letters.as_ref()
    }
}

impl PartialEq for LetterSet {
    fn eq(&self, other: &Self) -> bool {
        self.letters.eq(&other.letters)
    }
}

fn main() {
    let mut builder = trie_rs::TrieBuilder::new();
    builder.push(LetterSet::new(vec![b'a', b'b', b'c']));
    builder.push(LetterSet::new(vec![b'a', b'b']));
    let trie = builder.build();

    dbg!(trie.predictive_search(vec![b'a']));
}
