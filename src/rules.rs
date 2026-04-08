use crate::dice::Dice;
use crate::position::{generate_legal_positions, Position, State};
use crate::variants::VariantPosition;

pub trait PositionRules<const N: u8> {
    fn legal_positions(position: Position<N>, dice: &Dice) -> Vec<Position<N>>;
}

pub trait VariantRules {
    fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassicRules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoHitRules;

pub fn legal_positions_with<R, const N: u8>(position: Position<N>, dice: &Dice) -> Vec<Position<N>>
where
    R: PositionRules<N>,
{
    R::legal_positions(position, dice)
}

pub fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition> {
    <ClassicRules as VariantRules>::legal_positions(position, dice)
}

impl<const N: u8> PositionRules<N> for ClassicRules {
    fn legal_positions(position: Position<N>, dice: &Dice) -> Vec<Position<N>> {
        generate_legal_positions(position, dice)
    }
}

impl VariantRules for ClassicRules {
    fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition> {
        match position {
            VariantPosition::Backgammon(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Backgammon)
                .collect(),
            VariantPosition::Nackgammon(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Nackgammon)
                .collect(),
            VariantPosition::Longgammon(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Longgammon)
                .collect(),
            VariantPosition::Hypergammon(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Hypergammon)
                .collect(),
            VariantPosition::Hypergammon2(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Hypergammon2)
                .collect(),
            VariantPosition::Hypergammon4(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Hypergammon4)
                .collect(),
            VariantPosition::Hypergammon5(p) => generate_legal_positions(p, dice)
                .into_iter()
                .map(VariantPosition::Hypergammon5)
                .collect(),
        }
    }
}

impl<const N: u8> PositionRules<N> for NoHitRules {
    fn legal_positions(position: Position<N>, dice: &Dice) -> Vec<Position<N>> {
        let legal = generate_legal_positions(position, dice);
        legal
            .into_iter()
            .filter(|next| {
                // no-hits policy: opponent bar count must not increase after a move
                next.o_bar() <= position.o_bar()
            })
            .collect()
    }
}

impl VariantRules for NoHitRules {
    fn legal_positions(position: VariantPosition, dice: &Dice) -> Vec<VariantPosition> {
        match position {
            VariantPosition::Backgammon(p) => {
                <NoHitRules as PositionRules<15>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Backgammon)
                    .collect()
            }
            VariantPosition::Nackgammon(p) => {
                <NoHitRules as PositionRules<15>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Nackgammon)
                    .collect()
            }
            VariantPosition::Longgammon(p) => {
                <NoHitRules as PositionRules<15>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Longgammon)
                    .collect()
            }
            VariantPosition::Hypergammon(p) => {
                <NoHitRules as PositionRules<3>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Hypergammon)
                    .collect()
            }
            VariantPosition::Hypergammon2(p) => {
                <NoHitRules as PositionRules<2>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Hypergammon2)
                    .collect()
            }
            VariantPosition::Hypergammon4(p) => {
                <NoHitRules as PositionRules<4>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Hypergammon4)
                    .collect()
            }
            VariantPosition::Hypergammon5(p) => {
                <NoHitRules as PositionRules<5>>::legal_positions(p, dice)
                    .into_iter()
                    .map(VariantPosition::Hypergammon5)
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::Dice;
    use crate::position::generate_legal_positions;
    use crate::position::State;
    use crate::rules::{ClassicRules, NoHitRules, PositionRules, VariantRules};
    use crate::variants::{Variant, BACKGAMMON};

    #[test]
    fn classic_position_rules_match_existing_movegen() {
        let dice = Dice::new(6, 1);
        let from_rules = <ClassicRules as PositionRules<15>>::legal_positions(BACKGAMMON, &dice);
        let direct = generate_legal_positions(BACKGAMMON, &dice);
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

    #[test]
    fn no_hit_rules_filter_positions_that_put_opponent_on_bar() {
        let dice = Dice::new(6, 1);
        let legal = <NoHitRules as PositionRules<15>>::legal_positions(BACKGAMMON, &dice);
        assert!(!legal.is_empty());
        for next in legal {
            assert!(next.o_bar() <= BACKGAMMON.o_bar());
        }
    }
}
