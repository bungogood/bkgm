use crate::position::{Position, State};

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
        let raw = input.trim();
        let payload = raw.strip_prefix("XGID=").unwrap_or(raw);
        let parts: Vec<&str> = payload.split(':').collect();
        if parts.len() != 10 {
            return None;
        }

        let board = XgidBoard::parse(parts[0])?;
        let max_cube = parts[1].parse().ok()?;
        let match_length = parts[2].parse().ok()?;
        let rules = parts[3].parse().ok()?;
        let score_x = parts[4].parse().ok()?;
        let score_o = parts[5].parse().ok()?;
        let dice = parse_dice(parts[6])?;
        let move_flag = match parts[7] {
            "0" => false,
            "1" => true,
            _ => return None,
        };
        let cube_owner: i8 = parts[8].parse().ok()?;
        if !(-1..=1).contains(&cube_owner) {
            return None;
        }
        let cube_power = parts[9].parse().ok()?;

        Some(Self {
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

    pub fn format(self) -> String {
        let dice = match self.dice {
            XgidDice::Rolled(d1, d2) => format!("{}{}", d1, d2),
            XgidDice::DoubleOffered => "D".to_string(),
        };
        format!(
            "XGID={}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.board.format(),
            self.max_cube,
            self.match_length,
            self.rules,
            self.score_x,
            self.score_o,
            dice,
            if self.move_flag { 1 } else { 0 },
            self.cube_owner,
            self.cube_power,
        )
    }
}

impl XgidBoard {
    pub fn parse(input: &str) -> Option<Self> {
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
        Some(Self(out))
    }

    pub fn format(self) -> String {
        String::from_utf8(self.0.to_vec()).expect("board bytes must be ascii")
    }
}

impl<const N: u8> Position<N> {
    pub fn to_xgid_board(&self) -> String {
        let mut chars = [b'-'; 26];

        let x_bar = self.x_bar();
        if x_bar > 0 {
            chars[0] = (b'a' + (x_bar - 1)) as u8;
        }

        let o_bar = self.o_bar();
        if o_bar > 0 {
            chars[25] = (b'A' + (o_bar - 1)) as u8;
        }

        for i in 1..=24usize {
            let pip = 25 - i;
            let n = self.pip(pip);
            chars[i] = if n > 0 {
                b'a' + (n as u8 - 1)
            } else if n < 0 {
                b'A' + ((-n) as u8 - 1)
            } else {
                b'-'
            };
        }

        String::from_utf8(chars.to_vec()).expect("xgid board bytes must be ascii")
    }

    pub fn from_xgid_board(board: &str) -> Option<Self> {
        let board = XgidBoard::parse(board)?;
        let mut pips = [0i8; 26];

        for (i, ch) in board.0.iter().copied().enumerate() {
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
    use super::{Xgid, XgidDice};
    use crate::position::{Position, State};
    use crate::variants::{Variant, BACKGAMMON, HYPERGAMMON, LONGGAMMON};

    #[test]
    fn backgammon_start_board_encoding_matches_known_xgid_board() {
        assert_eq!(BACKGAMMON.to_xgid_board(), "-b----E-C---eE---c-e----B-");
    }

    #[test]
    fn xgid_board_roundtrip_position_id_samples() {
        let samples = [
            "4HPwATDgc/ABMA",
            "jGfkASjg8wcBMA",
            "zGbiIQgxH/AAWA",
            "zGbiIYCYD3gALA",
            "4HPwQQLgc/ABMA",
        ];

        for id in samples {
            let p = <Position<15> as State>::from_id(id).unwrap();
            let board = p.to_xgid_board();
            let parsed = Position::<15>::from_xgid_board(&board).unwrap();
            assert_eq!(p.position_id(), parsed.position_id());
        }
    }

    #[test]
    fn xgid_board_works_for_variants() {
        let p15 = LONGGAMMON;
        let p3 = HYPERGAMMON;

        let b15 = p15.to_xgid_board();
        let b3 = p3.to_xgid_board();

        let parsed15 = Variant::Longgammon.from_xgid_board(&b15).unwrap();
        let parsed3 = Variant::Hypergammon.from_xgid_board(&b3).unwrap();

        assert_eq!(parsed15.position_id(), p15.position_id());
        assert_eq!(parsed3.position_id(), p3.position_id());
    }

    #[test]
    fn full_xgid_parse_and_format_roundtrip() {
        let xgid = Xgid {
            board: super::XgidBoard::parse("-b----E-C---eE---c-e----B-").unwrap(),
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

        let encoded = xgid.format();
        let decoded = Xgid::parse(&encoded).unwrap();
        assert_eq!(decoded, xgid);
    }
}
