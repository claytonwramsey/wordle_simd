use std::{
    fs::File,
    io::{BufRead, BufReader},
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

const N_GRADES: usize = 0b1010101011;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        println!("wordle: usage: wordle <answers> <words>");
    }
    let answers: Vec<Word> = BufReader::new(File::open(&args[1])?)
        .lines()
        .map(|w| word_from_str(w?.as_bytes()).ok_or("invalid word".into()))
        .collect::<Result<Vec<Word>, Box<dyn std::error::Error>>>()?;

    let mut word_count = [0u16; N_GRADES];
    for s in BufReader::new(File::open(&args[2])?).lines() {
        let s = s?;
        let word = word_from_str(s.as_bytes()).ok_or("bad word")?;
        // let mut word_count = IntMap::with_capacity(answers.len());
        for &answer in &answers {
            word_count[grade(word, answer) as usize] += 1;
        }

        let info = word_count
            .iter()
            .filter(|&&n| n > 0)
            .map(|&n| (n as f64).log2() * n as f64)
            .sum::<f64>()
            / answers.len() as f64;

        word_count = [0; N_GRADES];
        println!("{s}: {info}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
