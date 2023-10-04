use std::fmt;

/// Contains a legal pair of dice (values between 1 and 6).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dice {
    Regular(RegularDice),
    Double(usize),
}

/// Contains two different values between 1 and six. `big` is bigger than `small`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RegularDice {
    pub big: usize,
    pub small: usize,
}

pub const ALL_SINGLES: [Dice; 15] = Dice::all_singles();
pub const ALL_21: [(Dice, f32); 21] = Dice::all_21();
pub const ALL_36: [Dice; 36] = Dice::all_36();
pub const ALL_1296: [(Dice, Dice); 1296] = Dice::all_1296();

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dice::Regular(dice) => write!(f, "({},{})", dice.big, dice.small),
            Dice::Double(die) => write!(f, "({},{})", die, die),
        }
    }
}

impl Dice {
    #[inline(always)]
    pub const fn new(die1: usize, die2: usize) -> Self {
        if die1 == die2 {
            Dice::Double(die1)
        } else {
            Dice::Regular(RegularDice::new(die1, die2))
        }
    }

    /// Contains all 21 unique possibilities of dice.
    const fn all_singles() -> [Dice; 15] {
        let mut dice = [Dice::Double(1); 15]; // Dummy values, will be replaced

        // for loops don't work with `const fn`
        let mut count = 0;
        let mut i = 0_usize;
        while i < 6 {
            let mut j = i + 1;
            while j < 6 {
                dice[count] = Dice::new(i + 1, j + 1);
                j += 1;
                count += 1;
            }
            i += 1;
        }
        dice
    }

    /// Contains all 21 unique possibilities of dice.
    const fn all_21() -> [(Dice, f32); 21] {
        let mut dice = [(Dice::Double(1), 0.0); 21]; // Dummy values, will be replaced

        // for loops don't work with `const fn`
        let mut count = 0;
        let mut i = 0_usize;
        while i < 6 {
            dice[count] = (Dice::Double(i + 1), 1.0);
            count += 1;
            let mut j = i + 1;
            while j < 6 {
                dice[count] = (Dice::new(i + 1, j + 1), 2.0);
                j += 1;
                count += 1;
            }
            i += 1;
        }
        dice
    }

    /// Contains all 36 possibilities of dice. Regular dice will appear twice.
    const fn all_36() -> [Dice; 36] {
        let mut dice = [Dice::Double(1); 36]; // Dummy values, will be replaced

        // for loops don't work with `const fn`
        let mut i = 0_usize;
        while i < 6 {
            let mut j = 0_usize;
            while j < 6 {
                dice[i * 6 + j] = Dice::new(i + 1, j + 1);
                j += 1;
            }
            i += 1;
        }
        dice
    }

    /// Contains all 1296 possibilities of the first two rolls. Regular dice will appear multiple times.
    const fn all_1296() -> [(Dice, Dice); 1296] {
        let dummy_value = (Dice::Double(1), Dice::Double(1));
        let mut dice = [dummy_value; 1296];
        let all_36 = ALL_36;

        // for loops don't work with `const fn`
        let array_length = 36;
        let mut i = 0_usize;
        while i < array_length {
            let mut j = 0_usize;
            while j < array_length {
                dice[i * array_length + j] = (all_36[i], all_36[j]);
                j += 1;
            }
            i += 1;
        }
        dice
    }
}

impl TryFrom<(usize, usize)> for Dice {
    type Error = &'static str;

    fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
        if value.0 < 1 || value.0 > 6 || value.1 < 1 || value.1 > 6 {
            Err("Dice values must be between 1 and 6.")
        } else {
            Ok(Dice::new(value.0, value.1))
        }
    }
}

impl RegularDice {
    #[inline(always)]
    pub(crate) const fn new(die1: usize, die2: usize) -> Self {
        let (big, small) = if die1 > die2 {
            (die1, die2)
        } else {
            (die2, die1)
        };
        Self { big, small }
    }
}

#[cfg(test)]
mod dice_tests {
    use crate::dice::Dice;

    #[test]
    fn all_36() {
        let all_36 = Dice::all_36();

        let smallest_double = Dice::new(1, 1);
        assert_eq!(all_36.iter().filter(|x| x == &&smallest_double).count(), 1);

        let biggest_double = Dice::new(6, 6);
        assert_eq!(all_36.iter().filter(|x| x == &&biggest_double).count(), 1);

        let regular = Dice::new(1, 6);
        assert_eq!(all_36.iter().filter(|x| x == &&regular).count(), 2);
    }
    #[test]
    fn all_1296() {
        let all_1296 = Dice::all_1296();

        let double_double = (Dice::new(1, 1), Dice::new(6, 6));
        assert_eq!(all_1296.iter().filter(|x| x == &&double_double).count(), 1);

        let double_regular = (Dice::new(2, 2), Dice::new(5, 4));
        assert_eq!(all_1296.iter().filter(|x| x == &&double_regular).count(), 2);

        let regular_double = (Dice::new(2, 3), Dice::new(5, 5));
        assert_eq!(all_1296.iter().filter(|x| x == &&regular_double).count(), 2);

        let regular_regular = (Dice::new(2, 3), Dice::new(3, 5));
        assert_eq!(
            all_1296.iter().filter(|x| x == &&regular_regular).count(),
            4
        );
    }
}
