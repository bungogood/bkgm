use crate::dice::Dice;
use crate::position::{Position, State};
use crate::variants::VariantPosition;

pub trait PositionRules<const N: u8> {
    fn legal_positions(position: Position<N>, dice: &Dice) -> Vec<Position<N>>;
}

pub trait VariantRules {
    fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassicRules;

impl<const N: u8> PositionRules<N> for ClassicRules {
    fn legal_positions(position: Position<N>, dice: &Dice) -> Vec<Position<N>> {
        position.possible_positions(dice)
    }
}

impl VariantRules for ClassicRules {
    fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition> {
        match position {
            VariantPosition::Backgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Backgammon)
                .collect(),
            VariantPosition::Nackgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Nackgammon)
                .collect(),
            VariantPosition::Longgammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Longgammon)
                .collect(),
            VariantPosition::Hypergammon(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon)
                .collect(),
            VariantPosition::Hypergammon2(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon2)
                .collect(),
            VariantPosition::Hypergammon4(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon4)
                .collect(),
            VariantPosition::Hypergammon5(p) => p
                .possible_positions(dice)
                .into_iter()
                .map(VariantPosition::Hypergammon5)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::Dice;
    use crate::position::State;
    use crate::rules::{ClassicRules, PositionRules, VariantRules};
    use crate::variants::{Variant, BACKGAMMON};

    #[test]
    fn classic_position_rules_match_existing_movegen() {
        let dice = Dice::new(6, 1);
        let from_rules = <ClassicRules as PositionRules<15>>::legal_positions(BACKGAMMON, &dice);
        let direct = BACKGAMMON.possible_positions(&dice);
        assert_eq!(from_rules, direct);
    }

    #[test]
    fn classic_variant_rules_match_game_legal_positions() {
        let game = crate::Game::new(Variant::Backgammon);
        let dice = Dice::new(6, 1);
        let via_rules = <ClassicRules as VariantRules>::legal_positions(game.position(), &dice);
        let via_game = game.legal_positions(&dice);
        assert_eq!(via_rules, via_game);
    }
}
