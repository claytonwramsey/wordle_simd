#![feature(result_flattening)]

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let wfile = BufReader::new(File::open(&args[1])?);

    let max_n = wfile
        .lines()
        .map(|wr| {
            let w = wr.unwrap();
            let b = w.as_bytes();
            let count = b
                .iter()
                .copied()
                .map(|c| b.iter().filter(|&&c1| c1 == c).count())
                .max()
                .unwrap();
            (w, count)
        })
        .max_by_key(|&(_, n)| n)
        .unwrap();

    println!("max word: {}", max_n.0);

    Ok(())
}
