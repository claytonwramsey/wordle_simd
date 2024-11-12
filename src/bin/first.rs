#![feature(portable_simd)]

use std::{
    collections::HashMap,
    fs::File,
    hint::black_box,
    io::{BufRead, BufReader, Read},
    str,
    time::{Duration, Instant},
};

use wordle::word_from_str;

fn stopwatch<F: FnOnce() -> R, R>(f: F) -> (R, Duration) {
    let tic = Instant::now();
    let res = f();
    (res, Instant::now().duration_since(tic))
}

fn make_bench<W>(
    name: &str,
    to_word: &impl Fn(&str) -> W,
    entropy_after: &impl Fn(W, &[W]) -> f32,
) {
    let args = std::env::args().collect::<Vec<_>>();
    let mut words_file = String::new();
    BufReader::new(File::open(&args[2]).unwrap())
        .read_to_string(&mut words_file)
        .unwrap();
    let (answers, ans_conv_time) = stopwatch(|| {
        BufReader::new(File::open(&args[1]).unwrap())
            .lines()
            .map(|x| to_word(&x.unwrap()))
            .collect::<Vec<_>>()
    });
    // println!("{name}: convert answers: {ans_conv_time:?}");
    let (words, words_conv_time) =
        stopwatch(|| words_file.lines().map(to_word).collect::<Vec<_>>());
    // println!("{name}: convert words: {words_conv_time:?}");

    let (best_word_id, best_word_time) = stopwatch(|| {
        let mut best_ent = f32::INFINITY;
        let mut best_word_id = usize::MAX;

        for (i, w) in words.into_iter().enumerate() {
            let ent = entropy_after(w, &answers);
            if ent < best_ent {
                best_ent = ent;
                best_word_id = i;
            }
        }

        best_word_id
    });

    let _ = black_box((best_word_id, words_conv_time, ans_conv_time));

    println!("{name}: find best word: {best_word_time:?}");
    // println!(
    //     "{name}: gives best word {:?}",
    //     words_file.lines().nth(best_word_id).unwrap()
    // );
}

fn main() {
    make_bench("naive", &(|s| s.to_string()), &|w, solns| {
        let mut word_count = HashMap::<_, usize>::new();
        for soln in solns {
            *word_count
                .entry(wordle::naive::grade(&w, soln))
                .or_default() += 1;
        }
        word_count
            .into_values()
            .filter(|&n| n > 1)
            .map(|n| (n as f32).log2() * n as f32)
            .sum::<f32>()
            / solns.len() as f32
    });
    make_bench(
        "sensible",
        &(|s| std::array::from_fn(|i| s.as_bytes()[i])),
        &|w, solns| {
            let mut word_count = HashMap::<_, usize>::new();
            for &soln in solns {
                *word_count
                    .entry(wordle::sensible::grade(w, soln))
                    .or_default() += 1;
            }
            word_count
                .into_values()
                .filter(|&n| n > 1)
                .map(|n| (n as f32).log2() * n as f32)
                .sum::<f32>()
                / solns.len() as f32
        },
    );
    make_bench(
        "packed",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|w, solns| {
            let mut word_count = [0u16; wordle::N_GRADES];
            for &answer in solns {
                word_count[wordle::packed::grade(w, answer) as usize] += 1;
            }
            word_count
                .into_iter()
                .filter(|&n| n > 1)
                .map(|n| (n as f32).log2() * n as f32)
                .sum::<f32>()
                / solns.len() as f32
        },
    );
    make_bench(
        "squeeze",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|w, solns| {
            let mut word_count = [0u16; wordle::N_GRADES];
            for &answer in solns {
                word_count[wordle::squeeze::grade(w, answer) as usize] += 1;
            }
            word_count
                .into_iter()
                .filter(|&n| n > 1)
                .map(|n| (n as f32).log2() * n as f32)
                .sum::<f32>()
                / solns.len() as f32
        },
    );
    make_bench(
        "squeeze simd(x1)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<1>,
    );
    make_bench(
        "squeeze simd(x2)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<2>,
    );
    make_bench(
        "squeeze simd(x4)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<4>,
    );
    make_bench(
        "squeeze simd(x8)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<8>,
    );
    make_bench(
        "squeeze simd(x16)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<16>,
    );
    make_bench(
        "squeeze simd(x32)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<32>,
    );
    make_bench(
        "squeeze simd(x64)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &wordle::squeeze::entropy_after::<64>,
    );
}
