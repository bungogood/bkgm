use crate::dice::Dice;
use crate::position::{GamePhase, GameState};
use crate::variants::{Variant, VariantPosition};

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

    pub fn from_position_id(variant: Variant, id: &str) -> Option<Self> {
        variant
            .from_position_id(id)
            .map(|position| Self { variant, position })
    }

    pub fn position(&self) -> VariantPosition {
        self.position
    }

    pub fn set_position(&mut self, position: VariantPosition) -> Result<(), &'static str> {
        if position.variant() != self.variant {
            return Err("position variant does not match game variant");
        }
        self.position = position;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.position = self.variant.start_position();
    }

    pub fn legal_positions(&self, dice: &Dice) -> Vec<VariantPosition> {
        self.position.possible_positions(dice)
    }

    pub fn apply_nth_legal_position(
        &mut self,
        dice: &Dice,
        index: usize,
    ) -> Result<(), &'static str> {
        let legal = self.legal_positions(dice);
        if index >= legal.len() {
            return Err("legal move index out of bounds");
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
}
