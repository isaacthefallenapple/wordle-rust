use std::fmt::{self, Write as _};
use std::io::{stdout, Read, Write};

use error::InvalidInputError;
use words::Word;

// TODO: let users pass in their own word lists
mod error;
mod stats;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: let users pass this in
/// The maximum number of turns a player is allowed to take.
const TURN_LIMIT: usize = 6;

fn main() -> Result<()> {
    let mut random_state = words::Rand::default();
    let word = words::pick_random_word(&mut random_state);
    let mut board = Board::new(word);

    let mut won = false;
    while !won && board.turn() < TURN_LIMIT {
        // TODO: error handling
        print!("Your guess: ");
        stdout().flush()?;
        board.input = match read_input() {
            Ok(input) => input,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
        println!();

        won = board.score().is_win();

        println!("{}", board);
    }

    if won {
        println!("🎉🎊🥳");
    } else {
        println!("Sorry, the word was {}", board.word_as_str());
    }

    Ok(())
}

/// `read_input` reads one guess from stdin into `buf`. Clears `buf` in the process.
fn read_input() -> Result<Word> {
    // 5 letters + \n
    let mut buf = [0u8; 6];
    let n = std::io::stdin().read(&mut buf)?;

    if n < 6 {
        return Err(InvalidInputError::InputLength.into());
    }
    if *buf.last().unwrap() != b'\n' {
        let mut stdin = std::io::stdin().lock();
        // clear stdin for next guess
        while !buf.contains(&b'\n') {
            stdin.read(&mut buf)?;
        }
        return Err(InvalidInputError::InputLength.into());
    }

    if let Some(&c) = buf.iter().find(|c| !c.is_ascii()) {
        return Err(InvalidInputError::NonAscii(c).into());
    }

    // ok to unwrap here, size has been asserted
    let mut guess = Word::try_from(&buf[..5]).unwrap();
    guess.make_ascii_uppercase();

    Ok(guess)
}

/// The score of a single letter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
enum LetterScore {
    /// A letter that is not in the word at all.
    #[default]
    Wrong = 0,
    /// A letter that is in the word but at a different spot.
    InWord = 1,
    /// A letter both in the word and at the right spot.
    Right = 2,
}

impl LetterScore {
    const ALL: [Self; 3] = [Self::Wrong, Self::InWord, Self::Right];

    // this is obviously unfortunate compared to just assigning the variants directly
    // but should the variants ever need special values this will come in handy.
    #[cfg(test)]
    const fn variant(self) -> u8 {
        self as u8
    }

    const fn bg_color(self) -> u8 {
        match self {
            Self::Wrong => 100,
            Self::InWord => 43,
            Self::Right => 42,
        }
    }
}

/// Renders `word` to `w` given `score`. Uses ANSI escapes to color the letters.
fn render(mut w: impl fmt::Write, word: &Word, score: Score) -> fmt::Result {
    for (i, c) in word.iter().enumerate() {
        let color = score.get(i).bg_color();
        write!(w, "\x1b[30;{color}m{0}", *c as char)?;
    }
    write!(w, "\x1b[m")?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Score(u8);

impl Score {
    const PERFECT: Self = Score(242);

    fn is_win(self) -> bool {
        self == Self::PERFECT
    }

    fn set(&mut self, at: usize, score: LetterScore) {
        let pos = 3u8.pow(at as u32);
        self.0 += score as u8 * pos;
    }

    fn get(self, at: usize) -> LetterScore {
        let pos = 3u8.pow(at as u32);
        LetterScore::ALL[(self.0 / pos) as usize % 3]
    }
}

fn score(word: &Word, guess: &Word) -> Score {
    if word == guess {
        return Score::PERFECT;
    }

    // invalid ascii byte to use as a placeholder
    const SENTINEL: u8 = u8::MAX;
    let mut score = Score::default();
    let mut word = *word;

    // find correct letters
    for (i, (w, g)) in word.iter_mut().zip(guess).enumerate() {
        if w == g {
            score.set(i, LetterScore::Right);
            *w = SENTINEL;
        }
    }

    for (i, g) in guess.iter().enumerate() {
        if score.get(i) == LetterScore::Right {
            continue;
        }
        if let Some(w) = word.iter_mut().find(|w| *w == g) {
            score.set(i, LetterScore::InWord);
            *w = SENTINEL;
        }
    }

    score
}

#[derive(Default)]
struct Board {
    word: Word,
    input: Word,
    guesses: [(Word, Score); TURN_LIMIT],
    turn: usize,
}

impl Board {
    fn new(word: Word) -> Self {
        Self {
            word,
            ..Default::default()
        }
    }

    fn score(&mut self) -> Score {
        let score = score(&self.word, &self.input);
        self.guesses[self.turn] = (self.input, score);
        self.turn += 1;
        score
    }

    fn word_as_str(&self) -> &str {
        std::str::from_utf8(&self.word).unwrap()
    }

    fn turn(&self) -> usize {
        self.turn
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (word, score) in &self.guesses[0..self.turn] {
            render(&mut *f, word, *score).expect("OOM");
            f.write_char('\n')?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score() {
        use LetterScore::*;
        let table = [
            (
                b"words",
                b"birds",
                [Wrong, Wrong, Right, Right, Right].into(),
            ),
            (
                b"tests",
                b"stabs",
                [InWord, InWord, Wrong, Wrong, Right].into(),
            ),
            (b"cargo", b"gocar", [InWord; 5].into()),
            (b"cargo", b"cargo", [Right; 5].into()),
            (
                b"stark",
                b"lossy",
                [Wrong, Wrong, InWord, Wrong, Wrong].into(),
            ),
            (
                b"liege",
                b"liens",
                [Right, Right, Right, Wrong, Wrong].into(),
            ),
            (
                b"liege",
                b"litre",
                [Right, Right, Wrong, Wrong, Right].into(),
            ),
            (
                b"abcde",
                b"edcba",
                [InWord, InWord, Right, InWord, InWord].into(),
            ),
            (
                b"abcde",
                b"ccccc",
                [Wrong, Wrong, Right, Wrong, Wrong].into(),
            ),
            (
                b"abcde",
                b"ccxxx",
                [InWord, Wrong, Wrong, Wrong, Wrong].into(),
            ),
        ];

        for (w, g, expected) in table {
            let got = score(w, g);
            assert_eq!(got, expected);
            // assert_eq!(got, decompress(compress(&got)));
        }
    }

    type ScoreArray = [LetterScore; 5];

    impl From<ScoreArray> for Score {
        fn from(value: ScoreArray) -> Self {
            Score(compress(&value))
        }
    }

    /// `compress` compresses a `Score` into a single byte.
    /// This works out because log2(3^5) < 8.
    ///
    /// [`decompress`] reverses this process again.
    #[allow(unused)]
    fn compress(score: &ScoreArray) -> u8 {
        let mut compressed = 0;
        for s in score.iter().rev() {
            compressed *= 3;
            compressed += s.variant();
        }
        compressed
    }
}
