use crate::position::{Position, State};
use crate::{Variant, VariantPosition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xgid {
    pub board: XgidBoard,
    pub max_cube: u8,
    pub match_length: u16,
    pub rules: u8,
    pub score_x: u16,
    pub score_o: u16,
    pub dice: XgidDice,
    pub move_flag: bool,
    pub cube_owner: i8,
    pub cube_power: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XgidBoard([u8; 26]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XgidDice {
    Rolled(u8, u8),
    DoubleOffered,
}

impl Xgid {
    pub fn parse(input: &str) -> Option<Self> {
        parse(input)
    }

    pub fn format(self) -> String {
        format(self)
    }
}

impl XgidBoard {
    pub fn parse(input: &str) -> Option<Self> {
        parse_board(input)
    }

    pub fn format(self) -> String {
        format_board(self)
    }

    pub(crate) fn from_bytes(bytes: [u8; 26]) -> Self {
        Self(bytes)
    }

    pub(crate) fn bytes(self) -> [u8; 26] {
        self.0
    }
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

pub fn decode_board(variant: Variant, board: &str) -> Option<VariantPosition> {
    match variant {
        Variant::Backgammon => {
            decode_board_for_position::<15>(board).map(VariantPosition::Backgammon)
        }
        Variant::Nackgammon => {
            decode_board_for_position::<15>(board).map(VariantPosition::Nackgammon)
        }
        Variant::Longgammon => {
            decode_board_for_position::<15>(board).map(VariantPosition::Longgammon)
        }
        Variant::Hypergammon => {
            decode_board_for_position::<3>(board).map(VariantPosition::Hypergammon)
        }
        Variant::Hypergammon2 => {
            decode_board_for_position::<2>(board).map(VariantPosition::Hypergammon2)
        }
        Variant::Hypergammon4 => {
            decode_board_for_position::<4>(board).map(VariantPosition::Hypergammon4)
        }
        Variant::Hypergammon5 => {
            decode_board_for_position::<5>(board).map(VariantPosition::Hypergammon5)
        }
    }
}

pub fn parse(input: &str) -> Option<Xgid> {
    let raw = input.trim();
    let payload = raw.strip_prefix("XGID=").unwrap_or(raw);
    let mut parts = payload.split(':');
    let board = parse_board(parts.next()?)?;
    let max_cube = parts.next()?.parse().ok()?;
    let match_length = parts.next()?.parse().ok()?;
    let rules = parts.next()?.parse().ok()?;
    let score_x = parts.next()?.parse().ok()?;
    let score_o = parts.next()?.parse().ok()?;
    let dice = parse_dice(parts.next()?)?;
    let move_flag = match parts.next()? {
        "0" => false,
        "1" => true,
        _ => return None,
    };
    let cube_owner: i8 = parts.next()?.parse().ok()?;
    if !(-1..=1).contains(&cube_owner) {
        return None;
    }
    let cube_power = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(Xgid {
        board,
        max_cube,
        match_length,
        rules,
        score_x,
        score_o,
        dice,
        move_flag,
        cube_owner,
        cube_power,
    })
}

pub fn format(xgid: Xgid) -> String {
    let dice = match xgid.dice {
        XgidDice::Rolled(d1, d2) => format!("{}{}", d1, d2),
        XgidDice::DoubleOffered => "D".to_string(),
    };
    format!(
        "XGID={}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        format_board(xgid.board),
        xgid.max_cube,
        xgid.match_length,
        xgid.rules,
        xgid.score_x,
        xgid.score_o,
        dice,
        if xgid.move_flag { 1 } else { 0 },
        xgid.cube_owner,
        xgid.cube_power,
    )
}

pub fn parse_board(input: &str) -> Option<XgidBoard> {
    let bytes = input.as_bytes();
    if bytes.len() != 26 {
        return None;
    }
    let mut out = [b'-'; 26];
    for (idx, byte) in bytes.iter().enumerate() {
        match *byte {
            b'-' => out[idx] = b'-',
            b'a'..=b'p' => out[idx] = *byte,
            b'A'..=b'P' => out[idx] = *byte,
            _ => return None,
        }
    }
    Some(XgidBoard::from_bytes(out))
}

pub fn format_board(board: XgidBoard) -> String {
    std::str::from_utf8(&board.bytes())
        .expect("board bytes must be ascii")
        .to_owned()
}

pub fn encode_board_for_position<const N: u8>(position: Position<N>) -> String {
    let mut chars = [b'-'; 26];

    let x_bar = position.x_bar();
    if x_bar > 0 {
        chars[0] = b'a' + (x_bar - 1);
    }

    let o_bar = position.o_bar();
    if o_bar > 0 {
        chars[25] = b'A' + (o_bar - 1);
    }

    for (i, ch) in chars.iter_mut().enumerate().take(25).skip(1) {
        let pip = 25 - i;
        let n = position.pip(pip);
        *ch = if n > 0 {
            b'a' + (n as u8 - 1)
        } else if n < 0 {
            b'A' + ((-n) as u8 - 1)
        } else {
            b'-'
        };
    }

    std::str::from_utf8(&chars)
        .expect("xgid board bytes must be ascii")
        .to_owned()
}

pub fn decode_board_for_position<const N: u8>(board: &str) -> Option<Position<N>> {
    let board = parse_board(board)?;
    let mut pips = [0i8; 26];

    for (i, ch) in board.bytes().iter().copied().enumerate() {
        if i == 0 {
            if ch == b'-' {
                continue;
            }
            if !(b'a'..=b'p').contains(&ch) {
                return None;
            }
            pips[25] = (ch - b'a' + 1) as i8;
            continue;
        }
        if i == 25 {
            if ch == b'-' {
                continue;
            }
            if !(b'A'..=b'P').contains(&ch) {
                return None;
            }
            pips[0] = -((ch - b'A' + 1) as i8);
            continue;
        }

        let pip = 25 - i;
        pips[pip] = match ch {
            b'-' => 0,
            b'a'..=b'p' => (ch - b'a' + 1) as i8,
            b'A'..=b'P' => -((ch - b'A' + 1) as i8),
            _ => return None,
        };
    }

    Position::try_from(pips).ok()
}

fn parse_dice(raw: &str) -> Option<XgidDice> {
    if raw == "D" {
        return Some(XgidDice::DoubleOffered);
    }
    let bytes = raw.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let d1 = (bytes[0] as char).to_digit(10)? as u8;
    let d2 = (bytes[1] as char).to_digit(10)? as u8;
    if d1 > 6 || d2 > 6 {
        return None;
    }
    Some(XgidDice::Rolled(d1, d2))
}

#[cfg(test)]
mod tests {
    use super::{decode_board, encode_board, format, parse, Xgid, XgidDice};
    use crate::codecs::gnuid;
    use crate::{Game, Variant};

    #[test]
    fn xgid_board_roundtrip() {
        let game = Game::new(Variant::Backgammon);
        let board = encode_board(game.position());
        let parsed = decode_board(Variant::Backgammon, &board).expect("must decode board");
        assert_eq!(gnuid::encode(parsed), gnuid::encode(game.position()));
    }

    #[test]
    fn xgid_full_roundtrip() {
        let game = Game::new(Variant::Backgammon);
        let board = encode_board(game.position());
        let full = Xgid {
            board: super::XgidBoard::parse(&board).expect("valid board"),
            max_cube: 0,
            match_length: 0,
            rules: 1,
            score_x: 0,
            score_o: 0,
            dice: XgidDice::Rolled(6, 1),
            move_flag: true,
            cube_owner: 0,
            cube_power: 0,
        };
        let encoded = format(full);
        let decoded = parse(&encoded).expect("must parse xgid");
        assert_eq!(decoded, full);
    }
}
