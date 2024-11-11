type Word = [u8; 5];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Color {
    Black,
    Yellow,
    Green,
}

pub type Grade = [Color; 5];

pub fn grade(w: Word, soln: Word) -> Grade {
    let mut bank = [0u8; 256];
    let mut grade = [Color::Black; 5];

    for ((wc, sc), g) in w.into_iter().zip(soln).zip(&mut grade) {
        if wc == sc {
            *g = Color::Green;
        } else {
            bank[sc as usize] += 1;
        }
    }

    for (wc, g) in w.into_iter().zip(&mut grade) {
        if *g == Color::Black && bank[wc as usize] > 0 {
            bank[wc as usize] -= 1;
            *g = Color::Yellow;
        }
    }

    grade
}
