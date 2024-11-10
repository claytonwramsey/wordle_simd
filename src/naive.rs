use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Yellow,
    Green,
}

pub type Word = str;
pub type Grade = Vec<Color>;

pub fn grade(w: &Word, sol: &Word) -> Grade {
    assert_eq!(w.len(), sol.len());
    let mut bank: HashMap<char, usize> = HashMap::new();
    let mut grade = Vec::with_capacity(w.len());

    for (wc, sc) in w.chars().zip(sol.chars()) {
        if wc == sc {
            grade.push(Color::Green);
        } else {
            grade.push(Color::Black);
            *bank.entry(sc).or_default() += 1;
        }
    }

    for (wc, g) in w
        .chars()
        .zip(&mut grade)
        .filter(|(_, g)| **g == Color::Black)
    {
        if let Some(c) = bank.get_mut(&wc) {
            if *c > 0 {
                *c -= 1;
                *g = Color::Yellow;
            }
        }
    }

    grade
}

#[cfg(test)]
mod tests {
    use super::*;

    use Color::*;

    #[test]
    fn horsehorse() {
        let graded = grade("horse", "horse");
        assert_eq!(graded, vec![Color::Green; 5]);
    }

    #[test]
    fn roseshorse() {
        let graded = grade("roses", "horse");
        assert_eq!(graded, vec![Yellow, Green, Yellow, Yellow, Black]);
    }
}
