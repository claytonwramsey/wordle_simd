#![feature(portable_simd)]

use std::{
    array,
    time::{Duration, Instant},
};

pub type Grade = u16;
pub type Word = u32;

pub mod naive;
pub mod packed;
pub mod sensible;
pub mod squeeze;

const GREEN: u16 = 0b10;
const YELLOW: u16 = 0b01;
const BLACK: u16 = 0b00;

pub const N_GRADES: usize = 0b1010101011;

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

pub fn stopwatch<F: FnOnce() -> R, R>(f: F) -> (R, Duration) {
    let tic = Instant::now();
    let res = f();
    (res, Instant::now().duration_since(tic))
}
