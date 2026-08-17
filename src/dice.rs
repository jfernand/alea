use crate::SIZES;
use rand::{Rng, rng};
use std::fmt::{Debug, Formatter};
use std::ops::Rem;

pub trait Rollable {
    fn roll(&self) -> u8;
}
#[derive(Clone, Copy)]
pub struct Dice(u8);

impl Debug for Dice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => write!(f, "d0"),
            4 => write!(f, "d4"),
            6 => write!(f, "d6"),
            8 => write!(f, "d8"),
            10 => write!(f, "d10"),
            12 => write!(f, "d12"),
            20 => write!(f, "d20"),
            _ => write!(f, "???"),
        }
    }
}

impl Dice {
    pub(crate) fn new(value: u8) -> Self {
        assert!(SIZES.contains(&value));
        Self(value)
    }
}

impl Rollable for Dice {
    fn roll(&self) -> u8 {
        if self.0 == 0 {
            return 0;
        }
        rng()
            .random::<u8>()
            .rem(self.0)
            + 1u8
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DiceSet<const N: usize> {
    dice: [Dice; N],
}

impl<const N: usize> DiceSet<N> {
    fn new(dice: [Dice; N]) -> Self {
        Self { dice }
    }
}

impl<const N: usize> Rollable for DiceSet<N> {
    fn roll(&self) -> u8 {
        self.dice
            .iter()
            .map(|d| d.roll())
            .sum()
    }
}

#[derive(Clone,  Debug, Eq, PartialEq)]
pub enum ParseError {
    InvalidFormat,
    InvalidDiceSize(String),
    InvalidCount,
    WrongDiceCount,
    DiceSizeNotSupported,
}

impl<const N: usize> TryInto<DiceSet<N>> for &str {
    type Error = ParseError;

    fn try_into(self) -> Result<DiceSet<N>, Self::Error> {
        let parts: Vec<&str> = self
            .split('+')
            .map(|s| s.trim())
            .collect();
        dbg!(&parts);
        let mut all_dice = Vec::new();

        for part in parts {
            let dice_parts: Vec<&str> = part
                .split('d')
                .collect();
            if dice_parts.len() != 2 {
                return Err(ParseError::InvalidFormat);
            }
            dbg!(&dice_parts);

            let count: usize = dice_parts[0]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidCount)?;
            let size: u8 = dice_parts[1]
                .trim()
                .parse()
                .map_err(|_| ParseError::InvalidDiceSize(dice_parts[1].to_string()))?;

            if !SIZES.contains(&size) {
                return Err(ParseError::DiceSizeNotSupported);
            }

            for _ in 0..count {
                all_dice.push(Dice::new(size));
            }
        }

        if all_dice.len() != N {
            return Err(ParseError::WrongDiceCount);
        }

        let dice_array: [Dice; N] = all_dice
            .try_into()
            .map_err(|_| ParseError::WrongDiceCount)?;

        Ok(DiceSet::new(dice_array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_dice_creation() {
        for &size in &SIZES {
            let die = Dice::new(size);
            assert_eq!(die.0, size);
        }
    }

    // #[test]
    // #[should_panic]
    // fn test_invalid_dice_creation_zero() {
    //     Dice::new(0);
    // }

    #[test]
    #[should_panic]
    fn test_invalid_dice_creation_odd() {
        Dice::new(5);
    }

    #[test]
    #[should_panic]
    fn test_invalid_dice_creation_large() {
        Dice::new(100);
    }

    #[test]
    fn test_dice_roll_bounds() {
        for &size in &SIZES {
            let die = Dice::new(size);
            for _ in 0..500 {
                let val = die.roll();
                if size == 0 {
                    assert_eq!(val, 0);
                    continue;
                }
                assert!(
                    val >= 1 && val <= size,
                    "Roll {} out of bounds for d{}",
                    val,
                    size
                );
            }
        }
    }

    #[test]
    fn test_dice_debug_format() {
        assert_eq!(format!("{:?}", Dice::new(4)), "d4");
        assert_eq!(format!("{:?}", Dice::new(6)), "d6");
        assert_eq!(format!("{:?}", Dice::new(8)), "d8");
        assert_eq!(format!("{:?}", Dice::new(10)), "d10");
        assert_eq!(format!("{:?}", Dice::new(12)), "d12");
        assert_eq!(format!("{:?}", Dice::new(20)), "d20");
        assert_eq!(format!("{:?}", Dice(5)), "???");
    }

    #[test]
    fn test_dice_set_roll() {
        let dice_set = DiceSet::new([Dice::new(6), Dice::new(8)]);
        for _ in 0..500 {
            let sum = dice_set.roll();
            assert!(
                sum >= 2 && sum <= 14,
                "Sum {} out of bounds for d6 + d8",
                sum
            );
        }
    }

    #[test]
    fn test_parse_dice_set_single() {
        let result: Result<DiceSet<1>, _> = "1d0".try_into();
        dbg!(&result);
        assert!(result.is_ok());

        let result: Result<DiceSet<1>, _> = "1d6".try_into();
        assert!(result.is_ok());

        let result2: Result<DiceSet<3>, _> = "3d20".try_into();
        assert!(result2.is_ok());
    }

    #[test]
    fn test_parse_dice_set_multiple_types() {
        let result: Result<DiceSet<4>, _> = "1d4 + 2d6 + 1d20".try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_dice_set_whitespace_handling() {
        let result: Result<DiceSet<2>, _> = "  1d8   +  1d10  ".try_into();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_format() {
        let r1: Result<DiceSet<1>, _> = "justtext".try_into();
        assert_eq!(r1.err(), Some(ParseError::InvalidFormat));

        let r2: Result<DiceSet<1>, _> = "1-6".try_into();
        assert_eq!(r2.err(), Some(ParseError::InvalidFormat));

        let r3: Result<DiceSet<2>, _> = "1d6+2d8+".try_into();
        assert_eq!(r3.err(), Some(ParseError::InvalidFormat));

        let r4: Result<DiceSet<1>, _> = "1dd6".try_into();
        assert_eq!(r4.err(), Some(ParseError::InvalidFormat));

        let r5: Result<DiceSet<1>, _> = "1d2d3".try_into();
        assert_eq!(r5.err(), Some(ParseError::InvalidFormat));
    }

    #[test]
    fn test_parse_invalid_count() {
        let r1: Result<DiceSet<1>, _> = "ad6".try_into();
        assert_eq!(r1.err(), Some(ParseError::InvalidCount));

        let r2: Result<DiceSet<1>, _> = "-2d6".try_into();
        assert_eq!(r2.err(), Some(ParseError::InvalidCount));

        let r3: Result<DiceSet<1>, _> = "d6".try_into();
        assert_eq!(r3.err(), Some(ParseError::InvalidCount));
    }

    #[test]
    fn test_parse_invalid_dice_size() {
        let r1: Result<DiceSet<1>, _> = "1d5".try_into();
        assert_eq!(r1.err(), Some(ParseError::DiceSizeNotSupported));

        let r3: Result<DiceSet<1>, _> = "1d100".try_into();
        assert_eq!(r3.err(), Some(ParseError::DiceSizeNotSupported));

        let r4: Result<DiceSet<1>, _> = "1dabc".try_into();
        assert_eq!(r4.err(), Some(ParseError::InvalidDiceSize("abc".into())));
    }

    #[test]
    fn test_parse_wrong_dice_count() {
        let r1: Result<DiceSet<2>, _> = "1d6".try_into();
        assert_eq!(r1.err(), Some(ParseError::WrongDiceCount));

        let r2: Result<DiceSet<1>, _> = "2d6".try_into();
        assert_eq!(r2.err(), Some(ParseError::WrongDiceCount));

        let r3: Result<DiceSet<3>, _> = "1d4 + 1d6".try_into();
        assert_eq!(r3.err(), Some(ParseError::WrongDiceCount));
    }
}
