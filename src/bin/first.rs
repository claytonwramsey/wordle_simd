#![feature(portable_simd)]

use core::f32;
use std::{
    collections::HashMap,
    fs::File,
    hint::black_box,
    io::{BufRead, BufReader, Read},
    str,
    thread::{available_parallelism, scope},
};

use wordle::{stopwatch, word_from_str};

fn make_bench<W: Send + Sync, F: Fn(&W, &[W]) -> f32 + Send + Sync>(
    name: &str,
    to_word: &impl Fn(&str) -> W,
    entropy_after: &F,
    n_threads: usize,
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
        let chunk_size = words.len().div_ceil(n_threads);
        scope(|s| {
            let handles: Vec<_> = words
                .chunks(chunk_size)
                .enumerate()
                .map(|(j, c)| {
                    let answers = answers.as_ref();
                    s.spawn(move || {
                        let mut best_ent = f32::INFINITY;
                        let mut best_word_id = usize::MAX;
                        for (i, w) in c.iter().enumerate() {
                            let ent = entropy_after(w, answers);
                            if ent < best_ent {
                                best_ent = ent;
                                best_word_id = i;
                            }
                        }
                        (best_ent, best_word_id + j * chunk_size)
                    })
                })
                .collect();

            let mut best_ent = f32::INFINITY;
            let mut best_id = usize::MAX;
            for handle in handles {
                let (ent, id) = handle.join().unwrap();
                if ent < best_ent {
                    best_ent = ent;
                    best_id = id;
                }
            }

            (best_ent, best_id)
        })
    });

    let _ = black_box((best_word_id, words_conv_time, ans_conv_time));

    println!("{name}: find best word: {best_word_time:?}");
    // println!(
    //     "{name}: gives best word {:?}",
    //     words_file.lines().nth(best_word_id.1).unwrap()
    // );
}

fn main() {
    make_bench(
        "naive",
        &(|s| s.to_string()),
        &|w, solns| {
            let mut word_count = HashMap::<_, usize>::new();
            for soln in solns {
                *word_count.entry(wordle::naive::grade(w, soln)).or_default() += 1;
            }
            word_count
                .into_values()
                .filter(|&n| n > 1)
                .map(|n| (n as f32).log2() * n as f32)
                .sum::<f32>()
                / solns.len() as f32
        },
        1,
    );
    make_bench(
        "sensible",
        &(|s| std::array::from_fn(|i| s.as_bytes()[i])),
        &|&w, solns| {
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
        1,
    );
    make_bench(
        "packed",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, solns| {
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
        1,
    );
    make_bench(
        "squeeze",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, solns| {
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
        1,
    );
    make_bench(
        "squeeze simd(x1)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<1>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x2)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<1>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x4)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<4>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x8)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<8>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x16)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<16>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x32)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<32>(w, s),
        1,
    );
    make_bench(
        "squeeze simd(x64)",
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<64>(w, s),
        1,
    );

    const L: usize = 32;
    let n_threads = available_parallelism().unwrap().get();
    make_bench(
        &format!("squeeze simd parallel({n_threads}x{L})"),
        &|s| word_from_str(s.as_bytes()).unwrap(),
        &|&w, s| wordle::squeeze::entropy_after::<L>(w, s),
        n_threads,
    );
}
