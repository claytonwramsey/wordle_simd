#![feature(portable_simd)]

use std::{
    array,
    fs::File,
    io::{BufRead, BufReader},
    simd::{prelude::*, LaneCount, Simd, SupportedLaneCount},
    str,
};

type Grade = u16;
type Word = u32;

const GREEN: u16 = 0b10;
const YELLOW: u16 = 0b01;
const BLACK: u16 = 0b00;

fn grade(guess: Word, soln: Word) -> Grade {
    let mut yellow_bank = 0u128;
    let mut grade = 0u16;
    let mut guess2 = guess;
    let mut soln2 = soln;
    for _ in 0..5 {
        let matches_bottom_5 = (guess2 ^ soln2) & 0x1f == 0;

        if matches_bottom_5 {
            grade |= GREEN << 10;
        } else {
            let sc = soln2 & 0x1f;
            yellow_bank += 1 << (3 * sc);
        }
        grade >>= 2;
        guess2 >>= 5;
        soln2 >>= 5;
    }

    for i in 0..5 {
        let c = (guess >> (5 * i)) & 0x1f;
        if grade & (0b11 << (2 * i)) == BLACK {
            let nyellow = (yellow_bank >> (3 * c)) & 0b111;
            if nyellow > 0 {
                yellow_bank -= 1 << (3 * c);
                grade |= YELLOW << (2 * i);
            }
        }
    }

    grade
}

fn gradel<const L: usize>(words: Simd<Word, L>, solns: Simd<Word, L>) -> Simd<Grade, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    // split yellow bank since u128 not supported
    let mut yellow_lo = Simd::<u64, L>::splat(0);
    let mut yellow_hi = yellow_lo;
    let mut grade = Simd::<u16, L>::splat(0);
    let mut guess2 = words;
    let mut soln2 = solns;

    let twenty = Simd::splat(20);
    for _ in 0..5 {
        let matches_bottom_5 = ((guess2 ^ soln2) & Simd::splat(0x1f)).simd_eq(Simd::splat(0));
        grade |= matches_bottom_5
            .cast()
            .select(Simd::splat(GREEN << 10), Simd::splat(BLACK));
        let sc = soln2 & Simd::splat(0x1f);
        let is_first_20_letters = sc.simd_lt(twenty);
        yellow_lo += (matches_bottom_5 | !is_first_20_letters).cast().select(
            Simd::splat(0),
            Simd::splat(1) << (Simd::splat(3u64) * sc.cast()),
        );
        yellow_hi += (matches_bottom_5 | is_first_20_letters).cast().select(
            Simd::splat(0),
            Simd::splat(1) << (Simd::splat(3u64) * (sc - twenty).cast()),
        );
        grade >>= 2;
        guess2 >>= 5;
        soln2 >>= 5;
    }

    for i in 0..5 {
        let twenty = Simd::splat(20u64);
        let c = ((words >> Simd::splat(5 * i)) & Simd::splat(0x1f)).cast();
        let is_first_twenty = c.simd_lt(twenty);
        let offset_c = is_first_twenty.select(c, c - twenty);

        let needs_yellow = (grade & Simd::splat(0b11 << (2 * i)))
            .simd_eq(Simd::splat(BLACK))
            .cast();
        // dbg!(i, needs_yellow);
        let n_yellow = (is_first_twenty.select(yellow_lo, yellow_hi)
            >> (Simd::splat(3u64) * offset_c))
            & Simd::splat(0b111);
        // dbg!(i, n_yellow);
        let got_yellow = needs_yellow & (n_yellow.simd_gt(Simd::splat(0)));
        // dbg!(i, got_yellow);

        grade |= got_yellow
            .cast()
            .select(Simd::splat(YELLOW << (2 * i)), Simd::splat(0));

        yellow_lo -= (got_yellow & is_first_twenty)
            .select(Simd::splat(1) << (Simd::splat(3u64) * c), Simd::splat(0));

        yellow_hi -= (got_yellow & !is_first_twenty).select(
            Simd::splat(1) << (Simd::splat(3u64) * offset_c),
            Simd::splat(0),
        );
    }

    grade
}

fn word_from_str(s: &[u8]) -> Option<Word> {
    if s.len() != 5 {
        return None;
    }
    let mut w = 0u32;
    for (i, &c) in s.iter().enumerate() {
        if !c.is_ascii_lowercase() {
            return None;
        }
        w |= u32::from(c - b'a') << (5 * i);
    }
    Some(w)
}

#[allow(dead_code)]
fn str_from_word(word: Word) -> [u8; 5] {
    array::from_fn(|i| ((word >> (5 * i)) & 0x1f) as u8 + b'a')
}

const N_GRADES: usize = 0b1010101011;
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
    let initial_entropy = (answers.len() as f64).log2();
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
        word_bits_left.push((word, entropy_after(word, &answers)));
    }

    word_bits_left.sort_unstable_by(|&(_, e1), &(_, e2)| e1.partial_cmp(&e2).unwrap());

    let mut best_entropy = word_bits_left[0].1;
    let mut opener_value = Vec::with_capacity(words.len() * (words.len() - 1) / 2);
    let mut possible_solns: [Vec<Word>; N_GRADES] = array::from_fn(|_| Vec::new()); // map from grades to possible solns
    for (i, &(w0, el0)) in word_bits_left.iter().enumerate() {
        let (prefix, simds, suffix) = answers.as_simd();
        for &answer in prefix {
            possible_solns[grade(w0, answer) as usize].push(answer);
        }
        for &answer in suffix {
            possible_solns[grade(w0, answer) as usize].push(answer);
        }
        for &answer in simds {
            let grades: Simd<usize, L> = gradel(Simd::splat(w0), answer).cast();
            for (graded, answer) in grades.to_array().into_iter().zip(answer.to_array()) {
                possible_solns[graded].push(answer);
            }
        }

        for &(w1, el1) in &word_bits_left[..i] {
            if el0 - (initial_entropy - el1) >= best_entropy {
                // cannot get extra info out
                // we sorted the words, so we won't have any more anyway
                break;
            }

            let mut rem_entropy = 0.0;
            for possibles in possible_solns.iter().filter(|v| v.len() > 1) {
                rem_entropy += entropy_after(w1, possibles) * possibles.len() as f64
            }
            rem_entropy /= answers.len() as f64;
            if rem_entropy < best_entropy {
                best_entropy = rem_entropy;
            }
            println!(
                "{}, {}: {rem_entropy}",
                str::from_utf8(&str_from_word(w0)).unwrap(),
                str::from_utf8(&str_from_word(w1)).unwrap()
            );

            opener_value.push(((w0, w1), rem_entropy));
        }

        // clean up but don't free the memory
        for v in &mut possible_solns {
            v.clear();
        }
    }

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

fn entropy_after(word: Word, solns: &[Word]) -> f64 {
    let mut word_count = [0u16; N_GRADES];
    let (prefix, simds, suffix) = solns.as_simd();
    for &answer in prefix {
        word_count[grade(word, answer) as usize] += 1;
    }
    for &answer in suffix {
        word_count[grade(word, answer) as usize] += 1;
    }
    for &answer in simds {
        let grades: Simd<usize, L> = gradel(Simd::splat(word), answer).cast();
        for graded in grades.to_array() {
            word_count[graded] += 1;
        }
    }
    word_count
        .iter()
        .filter(|&&n| n > 0)
        .map(|&n| (n as f64).log2() * n as f64)
        .sum::<f64>()
        / solns.len() as f64
}

#[cfg(test)]
mod tests {
    use core::str;
    use std::array;

    use super::*;

    #[test]
    fn horsehorse() {
        let graded = grade(
            word_from_str(b"horse").unwrap(),
            word_from_str(b"horse").unwrap(),
        );
        println!("{graded:b}");
        assert_eq!(graded, 0b1010101010);
    }

    #[test]
    fn roseshorse() {
        let graded = grade(
            word_from_str(b"roses").unwrap(),
            word_from_str(b"horse").unwrap(),
        );
        println!("{graded:b}");
        assert_eq!(
            graded,
            YELLOW | (GREEN << 2) | (YELLOW << 4) | (YELLOW << 6) | (BLACK << 8)
        );
    }

    #[test]
    fn simd_4x() {
        let words = [
            word_from_str(b"roses").unwrap(),
            word_from_str(b"horse").unwrap(),
            word_from_str(b"roses").unwrap(),
            word_from_str(b"horse").unwrap(),
        ];
        let solns = [
            word_from_str(b"roses").unwrap(),
            word_from_str(b"roses").unwrap(),
            word_from_str(b"horse").unwrap(),
            word_from_str(b"horse").unwrap(),
        ];

        let words_simd = Simd::from_array(words);
        let solns_simd = Simd::from_array(solns);
        let seq_grades: [_; 4] = array::from_fn(|i| grade(words[i], solns[i]));
        let simd_grades = gradel(words_simd, solns_simd).to_array();

        for i in 0..4 {
            println!("seq: {:b}, simd: {:b}", seq_grades[i], simd_grades[i]);
            assert_eq!(seq_grades[i], simd_grades[i]);
        }
    }

    #[test]
    fn aahed() {
        let word = word_from_str(b"aahed").unwrap();
        let words = Simd::splat(word);
        let solns = Simd::from_array([5800643]);
        let seq_grades = solns.to_array().map(|soln| grade(word, soln));
        let simd_grades = gradel(words, solns).to_array();

        for i in 0..simd_grades.len() {
            println!(
                "word {}, soln {}",
                word,
                str::from_utf8(&str_from_word(solns[i])).unwrap()
            );
            println!("seq: {:b}, simd: {:b}", seq_grades[i], simd_grades[i]);
            assert_eq!(seq_grades[i], simd_grades[i]);
        }
    }
}
