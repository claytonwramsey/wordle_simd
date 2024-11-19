use crate::{Grade, Word, BLACK, GREEN, YELLOW};

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

#[cfg(test)]
mod tests {
    use crate::word_from_str;

    use super::*;

    #[test]
    fn horsehorse() {
        let graded = grade(
            word_from_str(b"horse").unwrap(),
            word_from_str(b"horse").unwrap(),
        );
        println!("{graded:b}");
        assert_eq!(graded, GREEN * 0b0101010101);
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
}
