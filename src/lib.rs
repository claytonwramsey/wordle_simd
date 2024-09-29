#![feature(portable_simd)]

use std::{
    array,
    simd::{prelude::*, LaneCount, Simd, SupportedLaneCount},
};

pub type Grade = u16;
pub type Word = u32;

const GREEN: u16 = 0b10;
const YELLOW: u16 = 0b01;
const BLACK: u16 = 0b00;

pub const N_GRADES: usize = 0b1010101011;

pub fn grade(guess: Word, soln: Word) -> Grade {
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

pub fn gradel<const L: usize>(words: Simd<Word, L>, solns: Simd<Word, L>) -> Simd<Grade, L>
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

pub fn word_from_str(s: &[u8]) -> Option<Word> {
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

pub fn str_from_word(word: Word) -> [u8; 5] {
    array::from_fn(|i| ((word >> (5 * i)) & 0x1f) as u8 + b'a')
}

pub fn entropy_after<const L: usize>(word: Word, solns: &[Word]) -> f32
where
    LaneCount<L>: SupportedLaneCount,
{
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
        .into_iter()
        .filter(|&n| n > 1)
        .map(|n| (n as f32).log2() * n as f32)
        .sum::<f32>()
        / solns.len() as f32
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
