#![feature(portable_simd)]

use std::{
    fs::File,
    hint::black_box,
    io::{BufRead, BufReader},
    simd::{LaneCount, SupportedLaneCount},
    thread::{available_parallelism, scope},
};

use wordle::{squeeze::entropy_after, stopwatch, word_from_str, Word};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 4 {
        println!("flamegraph_scaling: usage: flamegraph_scaling <answers> <words>");
    }

    let solns: Vec<Word> = BufReader::new(File::open(&args[1])?)
        .lines()
        .map(|w| word_from_str(w?.as_bytes()).ok_or("invalid word".into()))
        .collect::<Result<Vec<Word>, Box<dyn std::error::Error>>>()?;

    let words = BufReader::new(File::open(&args[2])?)
        .lines()
        .map(|w| word_from_str(w?.as_bytes()).ok_or("invalid word".into()))
        .collect::<Result<Vec<Word>, Box<dyn std::error::Error>>>()?;

    println!("{} words and {} solutions", words.len(), solns.len());

    benchtime::<8>(
        available_parallelism().map_or(1, |x| x.get()),
        &words,
        &solns,
    );
    Ok(())
}

fn benchtime<const L: usize>(nthreads: usize, words: &[Word], solns: &[Word]) -> std::time::Duration
where
    LaneCount<L>: SupportedLaneCount,
{
    let chunk_size = words.len().div_ceil(nthreads);

    let ((_, best_word_id), best_word_time) = stopwatch(|| {
        scope(|s| {
            let handles: Vec<_> = words
                .chunks(chunk_size)
                .enumerate()
                .map(|(j, c)| {
                    s.spawn(move || {
                        let mut best_ent = f32::INFINITY;
                        let mut best_word_id = usize::MAX;
                        for (i, &w) in c.iter().enumerate() {
                            let ent = entropy_after::<L>(w, solns);
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

    black_box(best_word_id);
    best_word_time
}
