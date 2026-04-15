use std::io::{self, BufRead, Write};

use crate::codecs::gnuid;
use crate::dice::Dice;
use crate::{Game, Variant};

pub trait UbgiEngine {
    fn id_name(&self) -> &'static str;
    fn id_version(&self) -> &'static str;
    fn id_author(&self) -> &'static str {
        "unknown"
    }
    fn on_ready(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn choose_move(&mut self, game: &Game, dice: Dice) -> Result<String, String>;
}

pub fn parse_variant(name: &str) -> Result<Variant, String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "backgammon" | "bg" => Ok(Variant::Backgammon),
        "nackgammon" | "nack" => Ok(Variant::Nackgammon),
        "longgammon" | "long" => Ok(Variant::Longgammon),
        "hypergammon" | "hyper" | "hypergammon3" => Ok(Variant::Hypergammon),
        "hypergammon2" | "hyper2" => Ok(Variant::Hypergammon2),
        "hypergammon4" | "hyper4" => Ok(Variant::Hypergammon4),
        "hypergammon5" | "hyper5" => Ok(Variant::Hypergammon5),
        _ => Err(format!("unknown variant: {name}")),
    }
}

pub fn variant_name(variant: Variant) -> &'static str {
    match variant {
        Variant::Backgammon => "backgammon",
        Variant::Nackgammon => "nackgammon",
        Variant::Longgammon => "longgammon",
        Variant::Hypergammon => "hypergammon",
        Variant::Hypergammon2 => "hypergammon2",
        Variant::Hypergammon4 => "hypergammon4",
        Variant::Hypergammon5 => "hypergammon5",
    }
}

pub fn parse_variant_setoption(cmd: &str) -> Option<Result<Variant, String>> {
    let mut parts = cmd.split_whitespace();
    let setoption = parts.next()?;
    let name_kw = parts.next()?;
    let option_name = parts.next()?;
    let value_kw = parts.next()?;
    let value = parts.next()?;

    if !setoption.eq_ignore_ascii_case("setoption")
        || !name_kw.eq_ignore_ascii_case("name")
        || !value_kw.eq_ignore_ascii_case("value")
        || !option_name.eq_ignore_ascii_case("variant")
        || parts.next().is_some()
    {
        return None;
    }

    Some(parse_variant(value))
}

pub fn run_stdio_loop(engine: &mut impl UbgiEngine) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut variant = Variant::Backgammon;
    let mut game = Game::new(variant);
    let mut dice: Option<Dice> = None;

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }

        if cmd == "ubgi" {
            reply(&mut stdout, &format!("id name {}", engine.id_name()));
            reply(&mut stdout, &format!("id author {}", engine.id_author()));
            reply(&mut stdout, &format!("id version {}", engine.id_version()));
            reply(
                &mut stdout,
                "option name Variant type combo default backgammon var backgammon var nackgammon var longgammon var hypergammon var hypergammon2 var hypergammon4 var hypergammon5",
            );
            reply(&mut stdout, "ubgiok");
            continue;
        }

        if cmd == "isready" {
            match engine.on_ready() {
                Ok(()) => reply(&mut stdout, "readyok"),
                Err(err) => reply(&mut stdout, &format!("error internal isready_failed {err}")),
            }
            continue;
        }

        if cmd == "newgame" {
            game = Game::new(variant);
            dice = None;
            continue;
        }

        if let Some(parsed_variant) = parse_variant_setoption(cmd) {
            match parsed_variant {
                Ok(v) => {
                    variant = v;
                    game = Game::new(variant);
                }
                Err(_) => reply(&mut stdout, "error bad_argument variant"),
            }
            continue;
        }

        if let Some(id) = cmd.strip_prefix("position gnubgid ") {
            match gnuid::decode(variant, id.trim()) {
                Some(pos) => {
                    let _ = game.set_position(pos);
                }
                None => reply(&mut stdout, "error bad_argument invalid_position"),
            }
            continue;
        }

        if cmd == "position xgid" || cmd.starts_with("position xgid ") {
            reply(&mut stdout, "error unsupported_feature position_xgid");
            continue;
        }

        if let Some(rest) = cmd.strip_prefix("dice ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 2 {
                reply(&mut stdout, "error bad_argument dice");
                continue;
            }
            let d1 = parts[0].parse::<usize>();
            let d2 = parts[1].parse::<usize>();
            match (d1, d2) {
                (Ok(a), Ok(b)) if (1..=6).contains(&a) && (1..=6).contains(&b) => {
                    dice = Some(Dice::new(a, b));
                }
                _ => reply(&mut stdout, "error bad_argument dice"),
            }
            continue;
        }

        if cmd == "go" || cmd == "go role chequer" {
            let Some(current_dice) = dice else {
                reply(&mut stdout, "error missing_context dice");
                continue;
            };
            match engine.choose_move(&game, current_dice) {
                Ok(mv) => reply(&mut stdout, &format!("bestmove {mv}")),
                Err(err) => reply(
                    &mut stdout,
                    &format!("error internal move_select_failed {err}"),
                ),
            }
            continue;
        }

        if cmd == "quit" {
            break;
        }

        reply(&mut stdout, "error unknown_command");
    }
}

fn reply(out: &mut impl Write, line: &str) {
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use crate::Variant;

    use super::{parse_variant, parse_variant_setoption, variant_name};

    #[test]
    fn parse_variant_aliases() {
        assert_eq!(parse_variant("bg").unwrap(), Variant::Backgammon);
        assert_eq!(parse_variant("hyper2").unwrap(), Variant::Hypergammon2);
    }

    #[test]
    fn parse_variant_setoption_variant_only() {
        assert_eq!(
            parse_variant_setoption("setoption name Variant value longgammon")
                .unwrap()
                .unwrap(),
            Variant::Longgammon
        );
        assert!(parse_variant_setoption("setoption name Threads value 1").is_none());
    }

    #[test]
    fn variant_name_roundtrip() {
        assert_eq!(variant_name(Variant::Nackgammon), "nackgammon");
    }
}
