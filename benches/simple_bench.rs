use criterion::{criterion_group, criterion_main, Bencher, Criterion, ParameterizedBenchmark};
use lacbd::SimpleFinder;
use regex::RegexSet;
use std::ops::Range;

// fn random_words(len: usize) -> String {
//     use rand::prelude::*;

//     let mut rng = SmallRng::from_seed([0; 16]);

//     let mut random_word = || {
//         (0..)
//             .map(|_| rng.gen::<char>())
//             .filter(|c| c.is_alphanumeric())
//             .take(6)
//             .collect::<String>()
//     };

//     (0..len)
//         .map(|_| random_word())
//         .collect::<Vec<_>>()
//         .join(" ")
// }

// fn string_word_ranges(words: &str) -> Vec<Range<usize>> {
//     fn has_alphanumeric(s: &str) -> bool {
//         s.chars().any(char::is_alphanumeric)
//     }

//     words
//         // .split_word_bound_indices() TODO: reimplement
//         .filter(|(_, s)| has_alphanumeric(s))
//         .map(|(idx, s)| idx..(idx + s.len()))
//         .collect()
// }

// fn select_word_ranges(words: &str, num: usize) -> Vec<&str> {
//     use rand::prelude::*;

//     let split_words = string_word_ranges(words);

//     let mut rng = SmallRng::from_seed([0; 16]);

//     fn random_sentence<'a>(
//         rng: &mut SmallRng,
//         split_words: &[Range<usize>],
//         words: &'a str,
//     ) -> &'a str {
//         let len = rng.gen_range(3, 7);
//         let first_idx = rng.gen_range(0, split_words.len());
//         let last_idx = if first_idx + len >= split_words.len() {
//             split_words.len() - 1
//         } else {
//             first_idx + len
//         };

//         let first_byte = split_words[first_idx].start;
//         let last_byte = split_words[last_idx].end;
//         &words[first_byte..last_byte]
//     };

//     (0..num)
//         .map(|_| random_sentence(&mut rng, &split_words, words))
//         .collect()
// }

// fn do_simple_finder(word_len: usize, set_len: usize, b: &mut Bencher) {
//     let words = random_words(word_len);
//     let sentences = select_word_ranges(&words, set_len);
//     let searcher = SimpleFinder::new(sentences.iter().enumerate().map(|(i, s)| (*s, i)));
//     b.iter(|| searcher.find_all_unique(&words));
// }

// fn do_regex(word_len: usize, set_len: usize, b: &mut Bencher) {
//     let words = random_words(word_len);
//     let sentences = select_word_ranges(&words, set_len);

//     let r = RegexSet::new(
//         sentences
//             .iter()
//             .map(|s| format!(r"(?i)\b(?:{})\b", regex::escape(s))),
//     )
//     .unwrap();

//     b.iter(|| r.matches(&words));
// }

fn do_cracklib_finder(b: &mut Bencher) {
    use std::fs::File;
    use std::io::{prelude::*, BufReader};

    let f = File::open("/usr/share/dict/cracklib-small").expect("cracklib-small exists");
    let reader = BufReader::new(f);

    let mut lines = Vec::new();

    for line in reader.lines() {
        lines.push(line.unwrap());
    }

    let finder = SimpleFinder::new(lines.iter().map(|s| (s.as_ref(), ())));

    b.iter(|| finder.find_all_unique("cafécafé café café"));
}

// fn bench_set_length(c: &mut Criterion) {
//     c.bench(
//         "set_length",
//         ParameterizedBenchmark::new(
//             "SimpleFinder",
//             |b, &len| do_simple_finder(100, len, b),
//             (10..10000).step_by(1000),
//         )
//         .with_function("Regex", |b, &len| do_regex(1000, len, b)),
//     );
// }

// fn bench_haystack_length(c: &mut Criterion) {
//     c.bench(
//         "haystack_length",
//         ParameterizedBenchmark::new(
//             "SimpleFinder",
//             |b, &len| do_simple_finder(len, 10, b),
//             (10..10000).step_by(1000),
//         )
//         .with_function("Regex", |b, &len| do_regex(len, 100, b)),
//     );
// }

fn bench_cracklib(c: &mut Criterion) {
    c.bench_function(
        "cracklib_bench",
        do_cracklib_finder
    );
}

// criterion_group!(benches, bench_set_length, bench_haystack_length);
criterion_group!(benches, bench_cracklib);
criterion_main!(benches);
