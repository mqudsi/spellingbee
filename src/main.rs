use permutator::copy::Combination;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const MIN_LEN: usize = 4;
const DICT_FILE: &'static str = "/usr/share/dict/words";

fn factor(s: &str) -> Vec<u8> {
    let mut factors: Vec<u8> = s
        .chars()
        .map(|c| c.to_ascii_lowercase() as u8)
        .collect();
    factors.sort_unstable();
    factors.dedup();

    factors
}

/// Generate a list of list of all factors that can be found in a factor.
fn subsets(factors: &[u8], min_len: usize) -> Vec<Vec<u8>> {
    let mut subsets: Vec<Vec<u8>> = Vec::new();

    for len in min_len..=factors.len() {
        for word in factors.combination(len) {
            subsets.push(word);
        }
    }

    subsets
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    for arg in &args {
        assert!(arg.is_ascii(), "spellingbee only supports ascii input!");
    }
    let (pivot, letters) = match args.len() {
        1 => (None, &args[0]),
        2 if args[1].len() == 1 => (Some(args[1].chars().nth(0).unwrap()), &args[0]),
        _ => {
            eprintln!("USAGE: spellingbee LETTERS [PIVOT]");
            return Ok(());
        }
    };

    let factors = factor(letters);
    let mut map = HashMap::new();

    let file = File::open(DICT_FILE)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let word = line?;
        let factors = factor(&word);

        let words = map.entry(factors).or_insert(Vec::new());
        words.push(word);
    }

    let mut results = Vec::new();
    for subfactors in subsets(&factors, MIN_LEN) {
        let words = match map.get(&subfactors) {
            Some(words) => words,
            None => continue,
        };

        for word in words {
            if let Some(pivot) = pivot {
                if !word.contains(pivot) {
                    continue;
                }
            }
            // Don't print the words as we find them so that we
            // can sort by length before printing.
            // println!("{}", word);
            results.push(word);
        }
    }

    // Sort by length then alphabetically (not the other way around)
    results.sort_unstable_by(|a, b| {
        match a.len().cmp(&b.len()) {
            Ordering::Equal => a.cmp(b),
            result @ _ => result,
        }
    });

    let color_codes = ["\u{001b}[32m", "\u{001b}[0m"];
    for word in results.drain(..) {
        let anagram = factors.iter().all(|l| word.contains(*l as char));
        if anagram {
            println!("{}{word}{}", color_codes[0], color_codes[1]);
        } else {
            println!("{word}");
        }
    }

    Ok(())
}
