use crate::dice::Dice;
use crate::position::Position;
use crate::{State, Variant, VariantPosition};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FibsError {
    #[error("invalid FIBS board payload")]
    InvalidPayload,
    #[error("invalid FIBS turn '{0}'")]
    InvalidTurn(String),
    #[error("invalid FIBS bar/off counts")]
    InvalidCounts,
    #[error("invalid FIBS point token '{0}'")]
    InvalidPointToken(String),
    #[error("invalid FIBS position")]
    InvalidPosition,
    #[error("FIBS off counts do not match derived position")]
    OffCountMismatch,
}

pub fn encode_board(position: VariantPosition) -> String {
    match position {
        VariantPosition::Backgammon(p) => encode_board_for_position(p),
        VariantPosition::Nackgammon(p) => encode_board_for_position(p),
        VariantPosition::Longgammon(p) => encode_board_for_position(p),
        VariantPosition::Hypergammon(p) => encode_board_for_position(p),
        VariantPosition::Hypergammon2(p) => encode_board_for_position(p),
        VariantPosition::Hypergammon4(p) => encode_board_for_position(p),
        VariantPosition::Hypergammon5(p) => encode_board_for_position(p),
    }
}

pub fn decode_board(variant: Variant, text: &str) -> Result<VariantPosition, FibsError> {
    match variant {
        Variant::Backgammon => {
            decode_board_for_position::<15>(text).map(VariantPosition::Backgammon)
        }
        Variant::Nackgammon => {
            decode_board_for_position::<15>(text).map(VariantPosition::Nackgammon)
        }
        Variant::Longgammon => {
            decode_board_for_position::<15>(text).map(VariantPosition::Longgammon)
        }
        Variant::Hypergammon => {
            decode_board_for_position::<3>(text).map(VariantPosition::Hypergammon)
        }
        Variant::Hypergammon2 => {
            decode_board_for_position::<2>(text).map(VariantPosition::Hypergammon2)
        }
        Variant::Hypergammon4 => {
            decode_board_for_position::<4>(text).map(VariantPosition::Hypergammon4)
        }
        Variant::Hypergammon5 => {
            decode_board_for_position::<5>(text).map(VariantPosition::Hypergammon5)
        }
    }
}

pub fn encode_board_for_position<const N: u8>(position: Position<N>) -> String {
    let turn = if position.turn() { "x" } else { "o" };
    let bar = format!("x{},o{}", position.x_bar(), position.o_bar());
    let off = format!("x{},o{}", position.x_off(), position.o_off());
    let points = (1..=24)
        .rev()
        .map(|pip| {
            let n = position.pip(pip);
            if n > 0 {
                format!("x{}", n)
            } else if n < 0 {
                format!("o{}", -n)
            } else {
                "-".to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("turn={turn};bar={bar};off={off};points={points}")
}

pub fn decode_board_for_position<const N: u8>(text: &str) -> Result<Position<N>, FibsError> {
    let mut turn = None;
    let mut bar = None;
    let mut off = None;
    let mut points = None;

    for part in text.split(';') {
        let (k, v) = part.split_once('=').ok_or(FibsError::InvalidPayload)?;
        match k.trim() {
            "turn" => turn = Some(v.trim().to_string()),
            "bar" => bar = Some(v.trim().to_string()),
            "off" => off = Some(v.trim().to_string()),
            "points" => points = Some(v.trim().to_string()),
            _ => return Err(FibsError::InvalidPayload),
        }
    }

    let turn = turn.ok_or(FibsError::InvalidPayload)?;
    let bar = bar.ok_or(FibsError::InvalidPayload)?;
    let off = off.ok_or(FibsError::InvalidPayload)?;
    let points = points.ok_or(FibsError::InvalidPayload)?;

    let mut pips = [0i8; 26];

    let (x_bar, o_bar) = parse_two_counts(&bar)?;
    let (x_off_expected, o_off_expected) = parse_two_counts(&off)?;
    pips[25] = x_bar as i8;
    pips[0] = -(o_bar as i8);

    let mut tokens = points.split(',');
    for idx in 0..24 {
        let token = tokens.next().ok_or(FibsError::InvalidPayload)?;
        let pip = 24 - idx;
        pips[pip] = parse_point_token(token.trim())?;
    }
    if tokens.next().is_some() {
        return Err(FibsError::InvalidPayload);
    }

    let mut position = Position::<N>::try_from(pips).map_err(|_| FibsError::InvalidPosition)?;
    if turn == "o" {
        position = position.flip();
    } else if turn != "x" {
        return Err(FibsError::InvalidTurn(turn));
    }
    if position.x_off() != x_off_expected || position.o_off() != o_off_expected {
        return Err(FibsError::OffCountMismatch);
    }
    Ok(position)
}

pub fn normalize_move(text: &str) -> Option<String> {
    crate::codecs::move_text::normalize(text)
}

pub fn legal_moves(
    position: VariantPosition,
    dice: Dice,
) -> crate::codecs::move_text::MoveTextResult<Vec<(String, VariantPosition)>> {
    crate::codecs::move_text::legal(position, dice)
}

pub fn encode_move(
    position: VariantPosition,
    next_position: VariantPosition,
    dice: Dice,
) -> crate::codecs::move_text::MoveTextResult<String> {
    crate::codecs::move_text::encode(position, next_position, dice)
}

pub fn apply_move(position: VariantPosition, dice: Dice, text: &str) -> Option<VariantPosition> {
    crate::codecs::move_text::apply(position, dice, text)
}

fn parse_two_counts(raw: &str) -> Result<(u8, u8), FibsError> {
    let mut x = None;
    let mut o = None;
    for chunk in raw.split(',') {
        let chunk = chunk.trim();
        if let Some(rest) = chunk.strip_prefix('x') {
            x = Some(rest.parse::<u8>().map_err(|_| FibsError::InvalidCounts)?);
        } else if let Some(rest) = chunk.strip_prefix('o') {
            o = Some(rest.parse::<u8>().map_err(|_| FibsError::InvalidCounts)?);
        } else {
            return Err(FibsError::InvalidCounts);
        }
    }
    Ok((
        x.ok_or(FibsError::InvalidCounts)?,
        o.ok_or(FibsError::InvalidCounts)?,
    ))
}

fn parse_point_token(token: &str) -> Result<i8, FibsError> {
    if token == "-" {
        return Ok(0);
    }
    if let Some(rest) = token.strip_prefix('x') {
        return rest
            .parse::<i8>()
            .map_err(|_| FibsError::InvalidPointToken(token.to_string()));
    }
    if let Some(rest) = token.strip_prefix('o') {
        return rest
            .parse::<i8>()
            .map(|n| -n)
            .map_err(|_| FibsError::InvalidPointToken(token.to_string()));
    }
    Err(FibsError::InvalidPointToken(token.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{decode_board, encode_board};
    use crate::codecs::gnuid;
    use crate::position::Position;
    use crate::{Game, Variant};

    #[test]
    fn fibs_board_roundtrip_start_position() {
        let game = Game::new(Variant::Backgammon);
        let encoded = encode_board(game.position());
        let decoded = decode_board(Variant::Backgammon, &encoded).unwrap();
        assert_eq!(gnuid::encode(decoded), gnuid::encode(game.position()));
    }

    #[test]
    fn fibs_board_roundtrip_with_bars_and_off() {
        let mut pips = [0i8; 26];
        pips[25] = 2;
        pips[0] = -1;
        pips[8] = 3;
        pips[6] = 2;
        pips[19] = -2;
        pips[17] = -1;
        let position = Position::<15>::try_from(pips).expect("valid custom board");
        let variant_position = crate::VariantPosition::Backgammon(position);

        let encoded = encode_board(variant_position);
        let decoded = decode_board(Variant::Backgammon, &encoded).expect("must decode board");
        assert_eq!(gnuid::encode(decoded), gnuid::encode(variant_position));
    }
}
