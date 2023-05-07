use core::fmt;
use std::io::{self, stdout, Write};

use words::{Word, WORDS};

// TODO: let users pass in their own word lists
mod words;

// TODO: let users pass this in
/// The maximum number of turns a player is allowed to take.
const TURN_LIMIT: usize = 6;

fn main() {
    let mut rand = Rand::default();
    let word = pick_random_word(&mut rand);
    let mut board = Board::new(word);

    let mut input = String::new();
    let mut won = false;
    let mut turn = 0;
    while !won && turn < TURN_LIMIT {
        // TODO: error handling
        write!(stdout(), "Your guess: ").unwrap();
        stdout().flush().unwrap();
        board.input = read_input(&mut input).unwrap();
        writeln!(stdout()).unwrap();

        won = is_win(&board.score());

        writeln!(stdout(), "{}", board).unwrap();

        turn += 1;
    }

    if won {
        println!("🎉🎊🥳");
    } else {
        println!("Sorry, the word was {}", board.word_as_str());
    }
}

/// `read_input` reads one guess from stdin into `buf`. Clears `buf` in the process.
fn read_input(buf: &mut String) -> io::Result<Word> {
    buf.clear();
    std::io::stdin().read_line(buf)?;
    assert!(buf.is_ascii());
    // 5 letters + \n
    assert_eq!(buf.len(), 6);
    // ok to unwrap here, size has been asserted
    let mut guess = Word::try_from(&buf.as_bytes()[..5]).unwrap();
    guess.make_ascii_uppercase();

    Ok(guess)
}

fn pick_random_word(random_state: &mut Rand) -> Word {
    let index = random_state.sample() % words::WORD_COUNT as u64;
    *WORDS[index as usize]
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(u8)]
/// The score of a single letter.
enum LetterScore {
    #[default]
    /// A letter that is not in the word at all.
    Wrong,
    /// A letter that is in the word but at a different spot.
    InWord,
    /// A letter both in the word and at the right spot.
    Right,
}

impl LetterScore {
    const WRONG: u8 = 0;
    const IN_WORD: u8 = 1;
    const RIGHT: u8 = 2;

    // this is obviously unfortunate compared to just assigning the variants directly
    // but should the variants ever need special values this will come in handy.
    const fn variant(self) -> u8 {
        match self {
            Self::Wrong => Self::WRONG,
            Self::InWord => Self::IN_WORD,
            Self::Right => Self::RIGHT,
        }
    }
}

/// Renders `word` to `w` given `score`. Uses ANSI escapes to color the letters.
fn render(word: &Word, score: &Score, mut w: impl Write) {
    for (c, s) in word.iter().zip(score) {
        let color = match s {
            LetterScore::Wrong => 90,
            LetterScore::InWord => 93,
            LetterScore::Right => 32,
        };
        write!(w, "\x1b[{color}m{0}\x1b[m", *c as char).unwrap();
    }
}

type Score = [LetterScore; 5];

fn score(word: &Word, guess: &Word) -> Score {
    // invalid ascii byte to use as a placeholder
    const SENTINEL: u8 = u8::MAX;
    let mut score = Score::default();
    let mut word = *word;

    // find correct letters
    for (i, (w, g)) in word.iter_mut().zip(guess).enumerate() {
        if w == g {
            score[i] = LetterScore::Right;
            *w = SENTINEL;
        }
    }

    for (i, g) in guess.iter().enumerate() {
        if score[i] == LetterScore::Right {
            continue;
        }
        if let Some(w) = word.iter_mut().find(|w| *w == g) {
            score[i] = LetterScore::InWord;
            *w = SENTINEL;
        }
    }

    score
}

/// `compress` compresses a `Score` into a single byte.
/// This works out because log2(3^5) < 8.
///
/// [`decompress`] reverses this process again.
#[allow(unused)]
fn compress(score: &Score) -> u8 {
    let mut compressed = 0;
    for s in score.iter() {
        compressed *= 3;
        compressed += s.variant();
    }
    compressed
}

/// `decompress` reverses the [`compress`] process.
#[allow(unused)]
fn decompress(mut score: u8) -> Score {
    let mut decompressed = Score::default();
    for i in 0..decompressed.len() {
        let ls = match score % 3 {
            LetterScore::WRONG => LetterScore::Wrong,
            LetterScore::IN_WORD => LetterScore::InWord,
            LetterScore::RIGHT => LetterScore::Right,
            _ => unreachable!(),
        };
        decompressed[decompressed.len() - 1 - i] = ls;
        score /= 3;
    }
    decompressed
}

fn is_win(score: &Score) -> bool {
    score.iter().all(|&s| s == LetterScore::Right)
}

struct Board {
    word: Word,
    input: Word,
    guesses: Vec<(Word, Score)>,
}

impl Board {
    fn new(word: Word) -> Self {
        Self {
            word,
            input: Default::default(),
            guesses: Vec::with_capacity(6),
        }
    }

    fn score(&mut self) -> Score {
        let score = score(&self.word, &self.input);
        self.guesses.push((self.input, score));
        score
    }

    fn word_as_str(&self) -> &str {
        std::str::from_utf8(&self.word).unwrap()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: Vec<u8> = Vec::new();
        for (word, score) in &self.guesses {
            render(&word, &score, &mut s);
            s.push(b'\n');
        }

        f.write_str(&String::from_utf8(s).expect("rendered word isn't ascii"))
    }
}

/// `Rand` is a simple XorShift RNG.
struct Rand(u64);

impl Rand {
    // TODO: let users pass in a seed
    #[allow(unused)]
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn sample(&mut self) -> u64 {
        let x = &mut self.0;

        *x ^= *x << 13;
        *x ^= *x >> 17;
        *x ^= *x << 5;

        return *x;
    }
}

impl Default for Rand {
    fn default() -> Self {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos() as u64;
        Self(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_score() {
        use LetterScore::*;
        let table = [
            (b"words", b"birds", [Wrong, Wrong, Right, Right, Right]),
            (b"tests", b"stabs", [InWord, InWord, Wrong, Wrong, Right]),
            (b"cargo", b"gocar", [InWord; 5]),
            (b"stark", b"lossy", [Wrong, Wrong, InWord, Wrong, Wrong]),
            (b"liege", b"liens", [Right, Right, Right, Wrong, Wrong]),
            (b"liege", b"litre", [Right, Right, Wrong, Wrong, Right]),
            (b"abcde", b"edcba", [InWord, InWord, Right, InWord, InWord]),
            (b"abcde", b"ccccc", [Wrong, Wrong, Right, Wrong, Wrong]),
            (b"abcde", b"ccxxx", [InWord, Wrong, Wrong, Wrong, Wrong]),
        ];

        for (w, g, expected) in table {
            let got = score(w, g);
            assert_eq!(got, expected);
            assert_eq!(got, decompress(compress(&got)));
        }
    }
}
