use crate::dice::Dice;
use crate::position::Position;
use crate::{State, Variant, VariantPosition};

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

pub fn decode_board(variant: Variant, text: &str) -> Option<VariantPosition> {
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

pub fn decode_board_for_position<const N: u8>(text: &str) -> Option<Position<N>> {
    let mut turn = None;
    let mut bar = None;
    let mut off = None;
    let mut points = None;

    for part in text.split(';') {
        let (k, v) = part.split_once('=')?;
        match k.trim() {
            "turn" => turn = Some(v.trim().to_string()),
            "bar" => bar = Some(v.trim().to_string()),
            "off" => off = Some(v.trim().to_string()),
            "points" => points = Some(v.trim().to_string()),
            _ => return None,
        }
    }

    let turn = turn?;
    let bar = bar?;
    let off = off?;
    let points = points?;

    let mut pips = [0i8; 26];

    let (x_bar, o_bar) = parse_two_counts(&bar)?;
    let (x_off_expected, o_off_expected) = parse_two_counts(&off)?;
    pips[25] = x_bar as i8;
    pips[0] = -(o_bar as i8);

    let tokens: Vec<&str> = points.split(',').collect();
    if tokens.len() != 24 {
        return None;
    }
    for (idx, token) in tokens.iter().enumerate() {
        let pip = 24 - idx;
        pips[pip] = parse_point_token(token.trim())?;
    }

    let mut position = Position::<N>::try_from(pips).ok()?;
    if turn == "o" {
        position = position.flip();
    } else if turn != "x" {
        return None;
    }
    if position.x_off() != x_off_expected || position.o_off() != o_off_expected {
        return None;
    }
    Some(position)
}

pub fn normalize_move(text: &str) -> Option<String> {
    crate::codecs::move_text::normalize(text)
}

pub fn legal_moves(
    position: VariantPosition,
    dice: Dice,
) -> Result<Vec<(String, VariantPosition)>, String> {
    crate::codecs::move_text::legal(position, dice)
}

pub fn encode_move(
    position: VariantPosition,
    next_position: VariantPosition,
    dice: Dice,
) -> Result<String, String> {
    crate::codecs::move_text::encode(position, next_position, dice)
}

pub fn apply_move(position: VariantPosition, dice: Dice, text: &str) -> Option<VariantPosition> {
    crate::codecs::move_text::apply(position, dice, text)
}

fn parse_two_counts(raw: &str) -> Option<(u8, u8)> {
    let mut x = None;
    let mut o = None;
    for chunk in raw.split(',') {
        let chunk = chunk.trim();
        if let Some(rest) = chunk.strip_prefix('x') {
            x = rest.parse::<u8>().ok();
        } else if let Some(rest) = chunk.strip_prefix('o') {
            o = rest.parse::<u8>().ok();
        } else {
            return None;
        }
    }
    Some((x?, o?))
}

fn parse_point_token(token: &str) -> Option<i8> {
    if token == "-" {
        return Some(0);
    }
    if let Some(rest) = token.strip_prefix('x') {
        return rest.parse::<i8>().ok();
    }
    if let Some(rest) = token.strip_prefix('o') {
        return rest.parse::<i8>().ok().map(|n| -n);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{decode_board, encode_board};
    use crate::codecs::gnuid;
    use crate::{Game, Variant};

    #[test]
    fn fibs_board_roundtrip_start_position() {
        let game = Game::new(Variant::Backgammon);
        let encoded = encode_board(game.position());
        let decoded = decode_board(Variant::Backgammon, &encoded).unwrap();
        assert_eq!(gnuid::encode(decoded), gnuid::encode(game.position()));
    }
}
