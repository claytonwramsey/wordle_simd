#![feature(portable_simd)]

use core::str;
use std::{
    array,
    fs::File,
    io::{BufRead, BufReader},
    simd::{prelude::*, Simd},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    thread::{available_parallelism, scope},
};

use wordle::{
    squeeze::{grade, gradel},
    str_from_word, word_from_str, Word, N_GRADES,
};

const L: usize = 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("wordle: usage: wordle <answers> <words>");
    }
    let answers: Vec<Word> = BufReader::new(File::open(&args[1])?)
        .lines()
        .map(|w| word_from_str(w?.as_bytes()).ok_or("invalid word".into()))
        .collect::<Result<Vec<Word>, Box<dyn std::error::Error>>>()?;

    let mut words_human: Vec<[u8; 5]> = Vec::with_capacity(12948);
    let mut words = Vec::with_capacity(12948);
    let initial_entropy = (answers.len() as f32).log2();
    let mut word_bits_left = Vec::with_capacity(12948);

    for s in BufReader::new(File::open(&args[2])?).lines() {
        let s = s?;
        if s.len() != 5 || !s.bytes().all(|b| b.is_ascii_lowercase()) {
            return Err("bad word".into());
        }
        let sb = s.as_bytes();
        words_human.push(array::from_fn(|i| sb[i]));
        let word = unsafe { word_from_str(sb).unwrap_unchecked() };
        words.push(word);
        word_bits_left.push((word, wordle::squeeze::entropy_after::<L>(word, &answers)));
    }

    word_bits_left.sort_unstable_by(|&(_, e1), &(_, e2)| e1.partial_cmp(&e2).unwrap());

    let best_entropy = AtomicU64::new((word_bits_left[0].1 * 1e7) as u64);
    let next_start = AtomicUsize::new(0);

    println!("n_threads = {}", available_parallelism().unwrap());
    let do_work = |_tid: usize| {
        let mut opener_value = Vec::with_capacity(
            words.len() * (words.len() - 1) / (2 * available_parallelism().unwrap().get()),
        );
        let (prefix, simds, suffix) = answers.as_simd::<L>();
        let mut possible_solns: [Vec<Word>; N_GRADES] = array::from_fn(|_| Vec::new()); // map from grades to possible solns
        loop {
            let i = next_start.fetch_add(1, Ordering::Relaxed);
            if i >= words.len() {
                break;
            }
            let (w0, el0) = word_bits_left[i];
            // println!(
            //     "thread {tid} start word {}",
            //     str::from_utf8(&str_from_word(w0)).unwrap()
            // );

            for &answer in prefix {
                possible_solns[grade(w0, answer) as usize].push(answer);
            }
            for &answer in suffix {
                possible_solns[grade(w0, answer) as usize].push(answer);
            }
            for &answer in simds {
                let grades = gradel(Simd::splat(w0), answer);
                for (graded, answer) in grades.to_array().into_iter().zip(answer.to_array()) {
                    possible_solns[graded as usize].push(answer);
                }
            }

            for &(w1, el1) in &word_bits_left[..i] {
                if el0 - (initial_entropy - el1)
                    >= best_entropy.load(Ordering::Relaxed) as f32 / 1e7
                {
                    // cannot get extra info out
                    // we sorted the words, so we won't have any more anyway
                    break;
                }

                let mut rem_entropy = 0.0;
                for possibles in possible_solns.iter().filter(|v| v.len() > 1) {
                    rem_entropy +=
                        wordle::squeeze::entropy_after::<L>(w1, possibles) * possibles.len() as f32
                }
                rem_entropy /= answers.len() as f32;
                best_entropy.fetch_max((rem_entropy * 1e7) as u64, Ordering::Relaxed);
                // println!(
                //     "{}, {}: {rem_entropy}",
                //     str::from_utf8(&str_from_word(w0)).unwrap(),
                //     str::from_utf8(&str_from_word(w1)).unwrap()
                // );

                opener_value.push(((w0, w1), rem_entropy));
            }

            // clean up but don't free the memory
            for v in &mut possible_solns {
                v.clear();
            }
        }
        opener_value
    };

    let mut opener_value = Vec::with_capacity(words.len() * (words.len() - 1) / 2);
    scope(|s| {
        let handles = (0..available_parallelism().unwrap().get())
            .map(|tid| s.spawn(move || do_work(tid)))
            .collect::<Vec<_>>();
        for handle in handles {
            opener_value.extend(handle.join().unwrap());
        }
    });

    opener_value.sort_unstable_by(|&(_, e1), &(_, e2)| e1.partial_cmp(&e2).unwrap());
    println!("Top 10:");
    for ((w0, w1), entropy_left) in opener_value.into_iter().take(10) {
        println!(
            "{}, {}: {entropy_left}",
            str::from_utf8(&str_from_word(w0)).unwrap(),
            str::from_utf8(&str_from_word(w1)).unwrap()
        );
    }
    Ok(())
}
