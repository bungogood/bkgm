use crate::dice::MixedDice;
use crate::position::{Position, MOVES_CAPACITY, O_BAR, X_BAR};
use std::cmp::max;

impl<const N: u8> Position<N> {
    /// Returns all legal positions after rolling a double and then moving.
    /// The return values have not switched sides yet.
    #[inline]
    pub(super) fn all_positions_after_mixed_move(&self, dice: &MixedDice) -> Vec<Self> {
        debug_assert!(dice.big > dice.small);
        match self.pips[X_BAR] {
            0 => self.moves_with_0_checkers_on_bar(dice),
            1 => self.moves_with_1_checker_on_bar(dice),
            _ => self.moves_with_2_checkers_on_bar(dice),
        }
    }

    /// Mixed moves with exactly 1 checker on the bar.
    fn moves_with_1_checker_on_bar(&self, dice: &MixedDice) -> Vec<Self> {
        debug_assert!(self.pips[X_BAR] == 1);

        let mut moves: Vec<Self> = Vec::with_capacity(MOVES_CAPACITY);
        let mut enter_big: Option<Self> = None;
        let mut enter_small: Option<Self> = None;

        if self.can_enter(dice.big) {
            let position = self.clone_and_enter_single_checker(dice.big);
            enter_big = Some(position);
            (dice.small + 1..X_BAR).for_each(|i| {
                if position.can_move_when_bearoff_is_legal(i, dice.small) {
                    let position = position.clone_and_move_single_checker(i, dice.small);
                    moves.push(position);
                }
            });
        }

        let different_outcomes =
            self.pips[X_BAR - dice.big] < 0 || self.pips[X_BAR - dice.small] < 0;
        if self.can_enter(dice.small) {
            let position = self.clone_and_enter_single_checker(dice.small);
            enter_small = Some(position);
            let range = if different_outcomes {
                #[allow(clippy::reversed_empty_ranges)]
                (dice.big + 1..X_BAR).chain(1..0)
            } else {
                // In case move a single checker with both dice, we don't want to count that twice.
                (dice.big + 1..X_BAR - dice.small).chain(X_BAR - dice.small + 1..X_BAR)
            };
            range.for_each(|i| {
                if position.can_move_when_bearoff_is_legal(i, dice.big) {
                    let position = position.clone_and_move_single_checker(i, dice.big);
                    moves.push(position);
                }
            });
        }

        if moves.is_empty() {
            if let Some(position) = enter_big {
                moves.push(position);
            } else if let Some(position) = enter_small {
                moves.push(position);
            } else {
                moves.push(*self);
            }
        }
        moves
    }

    /// Mixed moves with no checkers on the bar.
    fn moves_with_0_checkers_on_bar(&self, dice: &MixedDice) -> Vec<Self> {
        debug_assert!(self.pips[X_BAR] == 0);

        // Let's try to find moves where both dice are used.
        let mut moves = self.two_checker_moves(dice);
        if moves.is_empty() {
            // No moves found with both dice used, so let's try the bigger die only.
            self.one_checker_moves(dice.big, &mut moves);
            if moves.is_empty() {
                // No moves found with the bigger die used, so let's try the smaller one.
                self.one_checker_moves(dice.small, &mut moves);
                if moves.is_empty() {
                    // The player can't move any checker, so we return the identical position.
                    moves.push(*self);
                }
            }
        }
        moves
    }

    /// All positions after moving a single checker once. If no move is possible it returns `None`.
    /// So if the return value is not `None`, the Vector is not empty.
    fn one_checker_moves(&self, die: usize, moves: &mut Vec<Self>) {
        debug_assert!(self.pips[X_BAR] == 0);

        (self.smallest_pip_to_check(die)..X_BAR).for_each(|i| {
            if self.can_move_when_bearoff_is_legal(i, die) {
                let position = self.clone_and_move_single_checker(i, die);
                moves.push(position);
            }
        });
    }

    // All moves with no checkers on the bar where two checkers can be moved.
    fn two_checker_moves(&self, dice: &MixedDice) -> Vec<Self> {
        debug_assert!(self.pips[X_BAR] == 0);

        let mut moves: Vec<Self> = Vec::with_capacity(MOVES_CAPACITY);

        // All moves where the `small` die is moved first
        (self.smallest_pip_to_check(dice.small)..X_BAR)
            .rev()
            .for_each(|i| {
                if self.can_move_when_bearoff_is_legal(i, dice.small) {
                    let position = self.clone_and_move_single_checker(i, dice.small);
                    (position.smallest_pip_to_check(dice.big)..i + 1)
                        .rev()
                        .for_each(|i| {
                            if position.can_move_when_bearoff_is_legal(i, dice.big) {
                                let position = position.clone_and_move_single_checker(i, dice.big);
                                moves.push(position);
                            }
                        });
                }
            });

        // All moves where the `big` die is moved first
        (self.smallest_pip_to_check(dice.big)..X_BAR).for_each(|i| {
            if self.can_move_when_bearoff_is_legal(i, dice.big) {
                let position = self.clone_and_move_single_checker(i, dice.big);
                let different_outcomes =
                    i >= dice.big && (self.pips[i - dice.big] < 0 || self.pips[i - dice.small] < 0);
                let smallest_pip = position.smallest_pip_to_check(dice.small);
                let range = if different_outcomes {
                    #[allow(clippy::reversed_empty_ranges)]
                    (smallest_pip..i).chain(0..0)
                } else {
                    // If we move a single checker with both dice, this position was already
                    // included above when `small` was moved first. So we omit it here.
                    let omitted_pip = i.saturating_sub(dice.big);
                    (smallest_pip..omitted_pip).chain(max(smallest_pip, omitted_pip + 1)..i)
                };
                range.for_each(|i| {
                    if position.can_move_when_bearoff_is_legal(i, dice.small) {
                        let position = position.clone_and_move_single_checker(i, dice.small);
                        moves.push(position);
                    }
                });
            }
        });

        moves
    }

    /// All moves (well, exactly one) when at least two checkers are on the bar.
    fn moves_with_2_checkers_on_bar(&self, dice: &MixedDice) -> Vec<Self> {
        debug_assert!(self.pips[X_BAR] > 1);

        let mut position = *self;
        if position.can_enter(dice.big) {
            position.enter_single_checker(dice.big);
        }
        if position.can_enter(dice.small) {
            position.enter_single_checker(dice.small);
        }
        vec![position]
    }

    fn can_enter(&self, die: usize) -> bool {
        debug_assert!(
            self.pips[X_BAR] > 0,
            "only call this function if x has checkers on the bar"
        );
        self.pips[X_BAR - die] > -2
    }

    fn clone_and_enter_single_checker(&self, die: usize) -> Self {
        let mut position = *self;
        position.enter_single_checker(die);
        position
    }

    fn enter_single_checker(&mut self, die: usize) {
        debug_assert!(
            self.pips[X_BAR] > 0,
            "only call this function if x has checkers on the bar"
        );
        debug_assert!(
            self.pips[X_BAR - die] > -2,
            "only call this function if x can enter"
        );
        self.pips[X_BAR] -= 1;
        if self.pips[X_BAR - die] == -1 {
            // hit opponent
            self.pips[X_BAR - die] = 1;
            self.pips[O_BAR] -= 1;
        } else {
            // no hitting
            self.pips[X_BAR - die] += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::MixedDice;
    use crate::pos;
    use crate::position::{O_BAR, X_BAR};

    // Two checkers on bar

    #[test]
    fn cannot_enter_with_two_checkers_on_bar() {
        // Given
        let position = pos!(x X_BAR:2, 10:2; o 22:2, 20:2);
        // When
        let resulting_positions =
            position.all_positions_after_mixed_move(&MixedDice { big: 5, small: 3 });
        // Then
        assert_eq!(resulting_positions, vec![position]);
    }

    #[test]
    fn can_enter_bigger_die_with_two_on_the_bar() {
        // Given
        let position = pos!(x X_BAR:2, 10:2; o 22:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        let expected = pos!(x X_BAR:1, 20:1, 10:2; o 22:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn can_enter_smaller_die_with_two_on_the_bar() {
        // Given
        let position = pos!(x X_BAR:2, 10:2; o 22:1, 20:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        let expected = pos!(x X_BAR:1, 22:1, 10:2; o 20:2, O_BAR:1);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn can_enter_both_with_three_on_the_bar() {
        // Given
        let position = pos!(x X_BAR:3, 10:2; o 20:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        let expected = pos!(x X_BAR:1, 22:1, 20:1, 10:2; o O_BAR:1);
        assert_eq!(resulting_positions, vec![expected]);
    }

    // One checker on bar

    #[test]
    fn cannot_enter_with_one_checker_on_bar() {
        // Given
        let position = pos!(x X_BAR:1, 10:2; o 22:2, 20:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        assert_eq!(resulting_positions, vec![position]);
    }

    #[test]
    fn can_enter_with_bigger_die_but_no_other_movement() {
        // Given
        let position = pos!(x X_BAR:1, 10:2; o 22:2, 20:1, 17:2, 7:3);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        let expected = pos!(x 20:1, 10:2; o 22:2, 17:2, 7:3, O_BAR:1);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn can_enter_with_smaller_die_but_no_other_movement() {
        // Given
        let position = pos!(x X_BAR:1; o 19:2, 14:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(6, 5));
        // Then
        let expected = pos!(x 20:1; o 19:2, 14:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn could_enter_with_either_die_but_must_use_bigger_one() {
        // Given
        let position = pos!(x X_BAR:1; o 20:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected = pos!(x 22:1; o 20:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn only_entering_with_smaller_die_allows_two_checkers_to_move() {
        // Given
        let position = pos!(x X_BAR:1, 12:1; o 20:2, 10:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected = pos!(x 23:1, 9:1; o 20:2, 10:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn only_entering_with_bigger_die_allows_two_checkers_to_move() {
        // Given
        let position = pos!(x X_BAR:1, 12:1; o 20:2, 9:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected = pos!(x 22:1, 10:1; o 20:2, 9:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn entering_with_either_die_allowed_but_only_one_final_position() {
        // Given
        let position = pos!(x X_BAR:1; o 9:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected = pos!(x 20:1; o 9:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn final_position_but_different_move_because_die1_hits_opponent() {
        // Given
        let position = pos!(x X_BAR:1; o 22:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected1 = pos!(x 20:1; o O_BAR:1);
        let expected2 = pos!(x 20:1; o 22:1);
        assert_eq!(resulting_positions, vec![expected1, expected2]);
    }

    #[test]
    fn final_position_but_different_move_because_die2_hits_opponent() {
        // Given
        let position = pos!(x X_BAR:1; o 23:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 2));
        // Then
        let expected1 = pos!(x 20:1; o 23:1);
        let expected2 = pos!(x 20:1; o O_BAR:1);
        assert_eq!(resulting_positions, vec![expected1, expected2]);
    }

    // No checkers on bar

    #[test]
    fn cannot_user_either_die() {
        // Given
        let position = pos!(x 10:2, 2:3; o 8:2, 6:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 2));
        // Then
        assert_eq!(resulting_positions, vec![position]);
    }

    #[test]
    fn forced_only_smaller_die() {
        // Given
        let position = pos!(x 7:2; o 2:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 2));
        // Then
        let expected = pos!(x 7:1, 5:1; o 2:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn forced_smaller_die_first_then_bear_off() {
        // Given
        let position = pos!(x 8:1, 4:3; o 1:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 3));
        // Then
        let expected = pos!(x 5:1, 4:2; o 1:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn bigger_die_cannot_move_initially() {
        // Given
        let position = pos!(x 20:1; o 16:3);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 3));
        // Then
        let expected = pos!(x 13:1; o 16:3);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn smaller_first_allows_bear_off() {
        // Given
        let position = pos!(x 9:1, 5:1; o 20:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 3));
        // Then
        let expected1 = pos!(x 4:1, 2:1; o 20:2);
        let expected2 = pos!(x 5:1, 1:1; o 20:2);
        let expected3 = pos!(x 6:1; o 20:2);
        assert_eq!(resulting_positions, vec![expected2, expected3, expected1]);
    }

    #[test]
    fn could_bear_off_but_could_do_other_moves_as_well() {
        // Given
        let position = pos!(x 5:2, 4:3, 3:1; o 20:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 3));
        // Then
        let expected1 = pos!(x 4:3, 3:1, 2:1, 1:1; o 20:1);
        let expected2 = pos!(x 5:1, 4:2, 3:1, 1:2; o 20:1);
        let expected3 = pos!(x 5:1, 4:3, 1:1; o 20:1);
        let expected4 = pos!(x 5:1, 4:2, 3:1, 2:1; o 20:1);
        let expected5 = pos!(x 5:2, 4:1, 3:1, 1:1; o 20:1);
        let expected6 = pos!(x 5:2, 4:2; o 20:1);
        assert_eq!(
            resulting_positions,
            vec![expected1, expected4, expected5, expected6, expected3, expected2]
        );
    }

    #[test]
    fn only_one_move_if_order_is_not_important() {
        // Given
        let position = pos!(x 20:1; o 22:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 3));
        // Then
        let expected = pos!(x 13:1; o 22:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn only_one_move_in_home_board_if_order_is_not_important() {
        // Given
        let position = pos!(x 5:1; o 22:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(2, 1));
        // Then
        let expected = pos!(x 2:1; o 22:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn two_moves_if_bigger_die_hits_opponent() {
        // Given
        let position = pos!(x 10:1; o 6:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(4, 2));
        // Then
        let expected1 = pos!(x 4:1; o O_BAR:1);
        let expected2 = pos!(x 4:1; o 6:1);
        assert_eq!(resulting_positions, vec![expected2, expected1]);
    }

    #[test]
    fn two_moves_if_smaller_die_hits_opponent() {
        // Given
        let position = pos!(x 5:1; o 4:1);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(3, 1));
        // Then
        let expected1 = pos!(x 1:1; o 4:1);
        let expected2 = pos!(x 1:1; o O_BAR:1);
        assert_eq!(resulting_positions, vec![expected2, expected1]);
    }

    #[test]
    fn two_bear_offs_from_same_pip() {
        // Given
        let position = pos!(x 1:5; o 24:8);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(6, 4));
        // Then
        let expected = pos!(x 1:3; o 24:8);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn bear_off_from_same_pip_with_either_big_or_small_die() {
        // Given
        let position = pos!(x 2:1, 1:5; o);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(6, 1));
        // Then
        let expected1 = pos!(x 1:4; o);
        let expected2 = pos!(x 1:5; o);
        assert_eq!(resulting_positions, vec![expected2, expected1]);
    }

    #[test]
    fn use_smaller_die_from_bigger_pip_case_when_bigger_pip_in_last_quarter() {
        // Given
        let position = pos!(x 7:1, 6:3; o 2:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(5, 4));
        // Then
        let expected = pos!(x 6:2, 3:1, 1:1; o 2:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn use_smaller_die_from_bigger_pip_case_when_bigger_in_first_quarter() {
        // Given
        let position = pos!(x 24:1, 8:1, 6:5; o 20:4, 19:3, 18:4, 16:2, 4:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(6, 2));
        // Then
        let expected = pos!(x 22:1, 6:5, 2:1; o 20:4, 19:3, 18:4, 16:2, 4:2);
        assert_eq!(resulting_positions, vec![expected]);
    }

    #[test]
    fn bearoff_with_same_checker_after_moving_small_first() {
        // Given
        let position = pos!(x 9:1, 5:3, 3:2; o 24:10, 23:3, 1:2);
        // When
        let resulting_positions = position.all_positions_after_mixed_move(&MixedDice::new(6, 4));
        // Then
        let expected = pos!(x 5:3, 3:2; o 24:10, 23:3, 1:2);
        assert_eq!(resulting_positions, vec![expected]);
    }
}
