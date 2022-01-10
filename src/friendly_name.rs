use lazy_static::lazy_static;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub fn friendly_name<T>(t: T) -> &'static str
where
    T: Hash,
{
    lazy_static! {
        static ref CHARS: HashSet<char> = "0123456789 \t".chars().collect();
        static ref WORDLIST: String =
            String::from_utf8_lossy(include_bytes!("data/eff_short_wordlist_2_0.txt")).into_owned();
        static ref WORDS: Vec<&'static str> = WORDLIST
            .lines()
            .map(|l| l.trim_start_matches(|c: char| CHARS.contains(&c)))
            .collect();
    }
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    let hash = h.finish();
    let mut rng: StdRng = SeedableRng::seed_from_u64(hash);
    WORDS.choose(&mut rng).unwrap()
}
