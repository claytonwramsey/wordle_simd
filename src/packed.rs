use std::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdUint,
    LaneCount, Simd, SupportedLaneCount,
};

use crate::{Grade, Word, BLACK, GREEN, N_GRADES, YELLOW};

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

pub fn gradel<const L: usize>(words: Simd<Word, L>, solns: Simd<Word, L>) -> Simd<u32, L>
where
    LaneCount<L>: SupportedLaneCount,
{
    // split yellow bank since u128 not supported
    let mut yellows = [Simd::<u32, L>::splat(0); 3];
    let mut grade = Simd::splat(0);
    let mut guess2 = words;
    let mut soln2 = solns;

    let ten = Simd::splat(10);
    let twenty = Simd::splat(20);
    for _ in 0..5 {
        let matches_bottom_5 = ((guess2 ^ soln2) & Simd::splat(0x1f)).simd_eq(Simd::splat(0));
        grade |= matches_bottom_5
            .cast()
            .select(Simd::splat((GREEN as u32) << 10), Simd::splat(BLACK as u32));
        let sc = soln2 & Simd::splat(0x1f);
        let is_first_ten = sc.simd_lt(ten);
        let is_ten_twenty = !is_first_ten & sc.simd_lt(twenty);
        let is_last_ten = sc.simd_ge(twenty);
        yellows[0] += (!matches_bottom_5 & is_first_ten)
            .select(Simd::splat(1) << (Simd::splat(3) * sc), Simd::splat(0));
        yellows[1] += (!matches_bottom_5 & is_ten_twenty).select(
            Simd::splat(1) << (Simd::splat(3) * (sc - ten)),
            Simd::splat(0),
        );
        yellows[2] += (!matches_bottom_5 & is_last_ten).select(
            Simd::splat(1) << (Simd::splat(3) * (sc - twenty)),
            Simd::splat(0),
        );
        grade >>= 2;
        guess2 >>= 5;
        soln2 >>= 5;
    }

    for i in 0..5 {
        let c = ((words >> Simd::splat(5 * i)) & Simd::splat(0x1f)).cast();
        let is_first_ten = c.simd_lt(ten);
        let is_ten_twenty = !is_first_ten & c.simd_lt(twenty);
        let is_last_ten = c.simd_ge(twenty);
        let offset_c = is_first_ten.select(c, is_ten_twenty.select(c - ten, c - twenty));

        let needs_yellow = (grade & Simd::splat(0b11 << (2 * i)))
            .simd_eq(Simd::splat(BLACK as u32))
            .cast();
        let n_yellow = (is_first_ten
            .select(yellows[0], is_ten_twenty.select(yellows[1], yellows[2]))
            >> (Simd::splat(3) * offset_c))
            & Simd::splat(0b111);
        let got_yellow = needs_yellow & (n_yellow.simd_gt(Simd::splat(0)));

        grade |= got_yellow
            .cast()
            .select(Simd::splat((YELLOW as u32) << (2 * i)), Simd::splat(0));

        let subs = Simd::splat(1) << (Simd::splat(3) * offset_c);
        yellows[0] -= (got_yellow & is_first_ten).select(subs, Simd::splat(0));
        yellows[1] -= (got_yellow & is_ten_twenty).select(subs, Simd::splat(0));
        yellows[2] -= (got_yellow & is_last_ten).select(subs, Simd::splat(0));
    }

    grade
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

    use crate::{str_from_word, word_from_str};

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
            assert_eq!(seq_grades[i], simd_grades[i] as u16);
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
            assert_eq!(seq_grades[i], simd_grades[i] as u16);
        }
    }
}
