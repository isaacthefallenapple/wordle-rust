use std::{
    error::Error,
    fmt,
    io::{Read, Write},
};

// repr(C) for stable ABI
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    wins: [u32; 6],
    losses: u32,
}

impl Stats {
    pub fn record_win(&mut self, round: usize) {
        debug_assert!(round < 6);
        self.wins[round] += 1;
    }

    pub fn record_loss(&mut self) {
        self.losses += 1;
    }

    pub fn serialize(&self, mut w: impl Write) -> Result<(), Box<dyn Error>> {
        for win in self.wins {
            write!(w, "{} ", win)?;
        }
        writeln!(w, "{}\n", self.losses)?;

        Ok(())
    }

    pub fn deserialize(mut r: impl Read) -> Result<Self, Box<dyn Error>> {
        const MAX_DIGITS: usize = 10;
        let mut buf = [0u8; MAX_DIGITS * 7 + 7];
        let mut cursor = 0;
        let mut n = 0;
        while !buf[cursor - n..cursor].contains(&b'\n') {
            n = r.read(&mut buf[cursor..])?;
            assert!(n > 0);
            cursor += n;
        }
        let buf = &buf[..cursor];

        let mut wins = [0u32; 6];
        let mut start = 0;
        for win in wins.iter_mut() {
            let Some(end) = buf[start..].iter().position(|b| *b == b' ') else {todo!()};
            let end = start + end;
            let digits = std::str::from_utf8(&buf[start..end])?;
            *win = digits.parse()?;
            start = end + 1;
        }

        let Some(end) = buf[start..].iter().position(|b| *b == b'\n') else {todo!()};
        let digits = std::str::from_utf8(&buf[start..start + end])?;
        let losses = digits.parse()?;

        Ok(Stats { wins, losses })
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SCALE: f32 = 30.0;
        let max = self.wins.iter().copied().max().unwrap().max(self.losses) as f32;

        for win in self.wins {
            let proportion = win as f32 / max;
            let columns = proportion * SCALE;
            write!(f, "{win} | ")?;
            for _ in 0..columns as usize {
                write!(f, "\u{1fb0e}")?;
            }
            writeln!(f)?;
        }

        let proportion = self.losses as f32 / max;
        let columns = proportion * SCALE;
        write!(f, "{} | ", self.losses)?;
        for _ in 0..columns as usize {
            write!(f, "\u{1fb0e}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let des = Stats::deserialize(&b"0 0 0 0 0 0 0\n"[..]).unwrap();
        assert_eq!(
            des,
            Stats {
                wins: [0; 6],
                losses: 0,
            }
        );

        let des = Stats::deserialize(&b"1 2 3 4 5 6 7\n"[..]).unwrap();
        assert_eq!(
            des,
            Stats {
                wins: [1, 2, 3, 4, 5, 6],
                losses: 7,
            }
        );

        let des = Stats::deserialize(format!("{0} {0} {0} {0} {0} {0} {0}\n", u32::MAX).as_bytes())
            .unwrap();
        assert_eq!(
            des,
            Stats {
                wins: [u32::MAX; 6],
                losses: u32::MAX,
            }
        );
    }
}
