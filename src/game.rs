use crate::dice::Dice;
use crate::dice_gen::DiceGen;
use crate::position::{GamePhase, GameState};
use crate::rules::{ClassicRules, VariantRules};
use crate::variants::{Variant, VariantPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("position variant does not match game variant")]
    VariantMismatch,
    #[error("legal move index out of bounds")]
    LegalMoveIndexOutOfBounds,
    #[error("policy returned out-of-bounds legal move index")]
    PolicyIndexOutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Game {
    variant: Variant,
    position: VariantPosition,
}

impl Game {
    pub fn new(variant: Variant) -> Self {
        Self {
            variant,
            position: variant.start_position(),
        }
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn position(&self) -> VariantPosition {
        self.position
    }

    pub fn set_position(&mut self, position: VariantPosition) -> Result<(), GameError> {
        if position.variant() != self.variant {
            return Err(GameError::VariantMismatch);
        }
        self.position = position;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.position = self.variant.start_position();
    }

    pub fn legal_positions(&self, dice: &Dice) -> Vec<VariantPosition> {
        self.legal_positions_with::<ClassicRules>(dice)
    }

    pub fn legal_positions_with<R: VariantRules>(&self, dice: &Dice) -> Vec<VariantPosition> {
        R::legal_positions(self.position, dice)
    }

    pub fn apply_nth_legal_position(&mut self, dice: &Dice, index: usize) -> Result<(), GameError> {
        self.apply_nth_legal_position_with::<ClassicRules>(dice, index)
    }

    pub fn apply_nth_legal_position_with<R: VariantRules>(
        &mut self,
        dice: &Dice,
        index: usize,
    ) -> Result<(), GameError> {
        let legal = self.legal_positions_with::<R>(dice);
        if index >= legal.len() {
            return Err(GameError::LegalMoveIndexOutOfBounds);
        }
        self.position = legal[index];
        Ok(())
    }

    pub fn phase(&self) -> GamePhase {
        self.position.phase()
    }

    pub fn game_state(&self) -> GameState {
        self.position.game_state()
    }

    pub fn play_episode_with<R: VariantRules, D: DiceGen, P>(
        &mut self,
        dice_gen: &mut D,
        max_plies: usize,
        mut pick_index: P,
    ) -> Result<usize, GameError>
    where
        P: FnMut(VariantPosition, &Dice, &[VariantPosition]) -> usize,
    {
        for ply in 0..max_plies {
            let dice = if ply == 0 {
                dice_gen.roll_mixed()
            } else {
                dice_gen.roll()
            };

            let legal = self.legal_positions_with::<R>(&dice);
            if legal.is_empty() {
                return Ok(ply);
            }

            let index = pick_index(self.position, &dice, &legal);
            if index >= legal.len() {
                return Err(GameError::PolicyIndexOutOfBounds);
            }

            self.position = legal[index];
            if let GameState::GameOver(_) = self.position.game_state() {
                return Ok(ply + 1);
            }
        }

        Ok(max_plies)
    }

    pub fn play_episode<D: DiceGen, P>(
        &mut self,
        dice_gen: &mut D,
        max_plies: usize,
        pick_index: P,
    ) -> Result<usize, GameError>
    where
        P: FnMut(VariantPosition, &Dice, &[VariantPosition]) -> usize,
    {
        self.play_episode_with::<ClassicRules, D, P>(dice_gen, max_plies, pick_index)
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, GameError};
    use crate::dice::Dice;
    use crate::dice_gen::DiceGenMock;
    use crate::rules::ClassicRules;
    use crate::variants::Variant;

    #[test]
    fn play_episode_uses_policy_and_advances() {
        let mut game = Game::new(Variant::Backgammon);
        let mut dice = DiceGenMock::new(&[Dice::new(6, 1), Dice::new(5, 3), Dice::new(4, 2)]);

        let plies = game
            .play_episode_with::<ClassicRules, _, _>(&mut dice, 3, |_pos, _dice, _legal| 0)
            .unwrap();

        assert_eq!(plies, 3);
    }

    #[test]
    fn play_episode_fails_on_out_of_bounds_policy_index() {
        let mut game = Game::new(Variant::Backgammon);
        let mut dice = DiceGenMock::new(&[Dice::new(6, 1)]);

        let err = game
            .play_episode_with::<ClassicRules, _, _>(&mut dice, 1, |_pos, _dice, legal| legal.len())
            .unwrap_err();

        assert_eq!(err, GameError::PolicyIndexOutOfBounds);
    }
}
