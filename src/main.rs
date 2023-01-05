use permutator::copy::Combination;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

const MIN_LEN: usize = 4;
const DICT_FILE: &'static str = "/usr/share/dict/words";
const SERIALIZED: &'static str = "./factors.bin";

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
        2 if args[1].len() == 1 => (Some(args[1].chars().nth(0).unwrap() as u8), &args[0]),
        _ => {
            eprintln!("USAGE: spellingbee LETTERS [PIVOT]");
            return Ok(());
        }
    };

    let factors = factor(letters);

    let mut serialized;
    let mut map_backing = HashMap::new();
    let mut map = &map_backing as &dyn GenericStrSliceMap<_, _>;
    let serialized_path = Path::new(SERIALIZED);
    if serialized_path.exists() {
        serialized = Vec::new();
        let result = File::open(serialized_path)
            .and_then(|mut file| file.read_to_end(&mut serialized))
            .and_then(|_| unsafe {
                let local_map = rkyv::archived_root::<HashMap<Vec<u8>, Vec<String>>>(&serialized[..]);
                map = local_map as &dyn GenericStrSliceMap<_, _>;
                Ok(())
            });

        if let Err(err) = result {
            eprintln!("Error loading saved factors state: {}", err);
        }
    }
    if map.is_empty() {
        map_backing = generate_dict_factors()?;
        cache_factors(&map_backing, serialized_path)?;
        map = &map_backing;
    }

    let mut results = Vec::new();
    for subfactors in subsets(&factors, MIN_LEN) {
        if let Some(pivot) = pivot {
            if !subfactors.contains(&pivot) {
                continue;
            }
        }

        map.for_each_of(&subfactors, |results, word| {
            // Don't print the words as we find them so that we
            // can sort by length before printing.
            // println!("{}", word);
            results.push(word);
        }, &mut results);
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

fn generate_dict_factors() -> Result<HashMap<Vec<u8>, Vec<String>>, std::io::Error> {
    let mut map = HashMap::new();
    let file = File::open(DICT_FILE)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let word = line?;
        let factors = factor(&word);

        let words = map.entry(factors).or_insert(Vec::new());
        words.push(word);
    }
    Ok(map)
}

fn cache_factors(map: &HashMap<Vec<u8>, Vec<String>>, path: &Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let mut file = File::create(path)?;
    file.write_all(&rkyv::to_bytes::<_, 256>(map)
        .map_err(|err| {
            eprintln!("{err}");
            std::io::Error::new(std::io::ErrorKind::Other, "Error serialized data!")
        })?)
}

trait GenericStrSliceMap<'a, F: Fn(&mut P, &'a str), P>
{
    fn is_empty(&self) -> bool;
    fn for_each_of(&'a self, key: &[u8], callback: F, p: &mut P);
}

impl<'a, F: Fn(&mut P, &'a str), P> GenericStrSliceMap<'a, F, P> for HashMap<Vec<u8>, Vec<String>>
{
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn for_each_of(&'a self, key: &[u8], callback: F, p: &mut P) {
        let Some(items) = self.get(key) else {
            return;
        };
        for value in items {
            callback(&mut *p, value)
        }
    }
}

use rkyv::collections::ArchivedHashMap;
use rkyv::string::ArchivedString;
use rkyv::vec::ArchivedVec;
impl<'a, F: Fn(&mut P, &'a str), P> GenericStrSliceMap<'a, F, P> for ArchivedHashMap<ArchivedVec<u8>, ArchivedVec<ArchivedString>>
{
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn for_each_of(&'a self, key: &[u8], callback: F, p: &mut P) {
        let Some(items) = self.get(key) else {
            return
        };
        for i in 0..items.len() {
            callback(&mut *p, &items[i]);
        }
    }
}
