use dice::Dice;
use rand::{Rng, rng};
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use crate::dice::{DiceSet, Rollable};

mod dice;

#[derive(Clone)]
struct Rating<T: std::marker::Copy> {
    rated_item: Cell<T>,
    score: Cell<f64>,
}

impl<T: Debug + Copy> Debug for Rating<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rating")
            .field(
                "rated_item",
                &self
                    .rated_item
                    .get(),
            )
            .field("score", &self.score)
            .finish()
    }
}

impl<T: std::marker::Copy + Rollable> Rating<T> {
    fn new(rated_item: T, rating: f64) -> Self {
        Self {
            rated_item: Cell::new(rated_item),
            score: Cell::new(rating),
        }
    }

    // s is 400
    // sigma(r) = 1/(1+10^(-r/s)
    // rA,B = Ra - Rb
    // Ea = sigma(ra,b)
    // Ra update = Ra + (Ra - Rb) * (1 - 10^(-r/s))
    // Rb update = Rb + K * (Sb - Eb)
    // K is 16ish
    fn win(&self, opponent: &Rating<T>) {
        let ra = &self.score;
        let rb = &opponent.score;
        let ea = sigma(ra.get() - rb.get(), 400.0);
        let delta = 16.0 * (1.0 - ea);
        self.score
            .set(
                self.score
                    .get()
                    + delta,
            );
        opponent
            .score
            .set(
                opponent
                    .score
                    .get()
                    - delta,
            );
    }
}

fn sigma(r: f64, s: f64) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf(-r / s))
}

struct Spec {
    offset: u8,
    die: Vec<Dice>,
}

const SIZES: [u8; 7] = [0, 4, 6, 8, 10, 12, 20];

fn random_die() -> Dice {
    Dice::new(random_die_size())
}

fn random_die_size() -> u8 {
    loop {
        let value = rng().random_range(4..=20);
        if SIZES.contains(&value) {
            break value;
        }
    }
}

fn dice_power_set() -> Vec<Vec<u8>> {
    let mut out = Vec::with_capacity(1 << 8); // 256 subsets
    for mask in 0u16..(1u16 << 8) {
        let mut subset = Vec::new();
        for i in 0..SIZES.len() {
            println!("i: {}", i);
            if (mask & (1 << i)) != 0 {
                subset.push(SIZES[i] as u8);
            }
        }
        if !subset.is_empty() {
            out.push(subset);
        }
    }
    out
}

fn main() {
    let mut rng = rng();

    let mut dice = vec![
        // Rating::new(Dice::new(4), 1500.0),
        // Rating::new(Dice::new(8), 0.0),
        // Rating::new(Dice::new(10), 0.0),
        // Rating::new(Dice::new(6), 0.0),
        // Rating::new(Dice::new(12), 0.0),
        Rating::<DiceSet<2>>::new("1d8+1d0".try_into().unwrap(), 0.0),
        Rating::new("2d4".try_into().unwrap(), 0.0),
        Rating::new("2d6".try_into().unwrap(), 0.0),
        Rating::new("1d6+1d0".try_into().unwrap(), 0.0),
        Rating::new("1d4+1d0".try_into().unwrap(), 0.0),
        Rating::new("2d8".try_into().unwrap(), 0.0),
        Rating::new("1d10+1d0".try_into().unwrap(), 0.0),
    ];
    for _ in 0..10_000_000 {
        // let mut v: Vec<&Rating<Dice>> = dice.choose_multiple(&mut rng, 2).collect();
        // let a = v.get_mut(0).unwrap();
        // let b =v.get_mut(1).unwrap();
        let idx1 = rng.random_range(0..dice.len());
        let mut idx2 = rng.random_range(0..dice.len());
        while idx2 == idx1 {
            idx2 = rng.random_range(0..dice.len());
        }
        battle(&dice[idx1], &dice[idx2])
    }

    dice.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap()
    });
    for die in &dice {
        println!(
            "{:?} - score: {}",
            die.rated_item
                .get(),
            die.score
                .get()
        );
    }
}

// s is 400
// sigma(r) = 1/(1+10^(-r/s)
// rA,B = Ra - Rb
// Ea = sigma(ra,b)
// Ra update = Ra + (Ra - Rb) * (1 - 10^(-r/s))
// Rb update = Rb + K * (Sb - Eb)
// K is 16ish

fn battle<T>(a: &Rating<T>, b: &Rating<T>)
where
    T: std::marker::Copy + Rollable
{
    // println!("battle: {:?} {:?}", a, b);
    let a_roll = a
        .rated_item
        .get()
        .roll();
    let b_roll = b
        .rated_item
        .get()
        .roll();
    if a_roll > b_roll {
        a.win(b);
    } else if a_roll < b_roll {
        b.win(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_properties() {
        let s = 400.0;
        // Equal rating difference -> expected win probability 0.5
        let eq_prob = sigma(0.0, s);
        assert!((eq_prob - 0.5).abs() < 1e-6);

        // Symmetry: sigma(r) + sigma(-r) = 1.0
        for diff in [-1000.0, -400.0, -100.0, 0.0, 100.0, 400.0, 1000.0] {
            let sum = sigma(diff, s) + sigma(-diff, s);
            assert!((sum - 1.0).abs() < 1e-6, "Sum {} != 1.0 for diff {}", sum, diff);
        }

        // Expected score for +400 rating difference: 1 / (1 + 10^-1) = 1 / 1.1 ≈ 0.9090909
        let plus_400 = sigma(400.0, s);
        assert!((plus_400 - (10.0 / 11.0)).abs() < 1e-6);
    }

    #[test]
    fn test_rating_creation_and_debug() {
        let r = Rating::new(Dice::new(6), 1500.0);
        assert_eq!(r.score.get(), 1500.0);
        let debug_str = format!("{:?}", r);
        assert!(debug_str.contains("Rating"));
        assert!(debug_str.contains("1500"));
    }

    #[test]
    fn test_rating_win_conservation() {
        let a = Rating::new(Dice::new(6), 1500.0);
        let b = Rating::new(Dice::new(6), 1500.0);

        let initial_sum = a.score.get() + b.score.get();
        a.win(&b);

        // With delta = 16 * (1 - 0.5) = 8
        assert!((a.score.get() - 1508.0).abs() < 1e-6);
        assert!((b.score.get() - 1492.0).abs() < 1e-6);
        assert!(((a.score.get() + b.score.get()) - initial_sum).abs() < 1e-6);
    }

    #[test]
    fn test_random_die_size() {
        for _ in 0..100 {
            let size = random_die_size();
            assert!(SIZES.contains(&size), "Size {} not in SIZES", size);
        }
    }

    #[test]
    fn test_random_die() {
        for _ in 0..100 {
            let die = random_die();
            let roll = die.roll();
            assert!(roll >= 1 && roll <= 20);
        }
    }

    #[test]
    fn test_dice_power_set() {
        let power_set = dice_power_set();
        assert!(!power_set.is_empty());
        for subset in &power_set {
            assert!(!subset.is_empty());
            for &size in subset {
                assert!(SIZES.contains(&size));
            }
        }
    }

    #[test]
    fn test_battle_execution() {
        let a = Rating::new(Dice::new(20), 1000.0);
        let b = Rating::new(Dice::new(4), 1000.0);

        let initial_sum = a.score.get() + b.score.get();
        for _ in 0..50 {
            battle(&a, &b);
        }

        // Conservation of total rating holds across multiple battles
        assert!(((a.score.get() + b.score.get()) - initial_sum).abs() < 1e-6);
    }
}
