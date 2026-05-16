use std::io::{self, BufRead, Write};

use crate::codecs::gnuid;
use crate::dice::Dice;
use crate::{Game, Variant, VARIANTS};

pub enum OptionSpec {
    Integer {
        key: String,
        min: i64,
        max: i64,
        default: i64,
    },
    Boolean {
        key: String,
        default: bool,
    },
    Choice {
        key: String,
        choices: Vec<String>,
        default: String,
    },
    Text {
        key: String,
        default: String,
    },
}

impl OptionSpec {
    pub fn integer(key: &str, min: i64, max: i64, default: i64) -> Self {
        Self::Integer {
            key: key.to_string(),
            min,
            max,
            default,
        }
    }

    pub fn boolean(key: &str, default: bool) -> Self {
        Self::Boolean {
            key: key.to_string(),
            default,
        }
    }

    pub fn choice(key: &str, choices: &[&str], default: &str) -> Self {
        Self::Choice {
            key: key.to_string(),
            choices: choices.iter().map(|s| (*s).to_string()).collect(),
            default: default.to_string(),
        }
    }

    pub fn text(key: &str, default: &str) -> Self {
        Self::Text {
            key: key.to_string(),
            default: default.to_string(),
        }
    }

    pub fn to_key_line(&self) -> String {
        match self {
            Self::Integer {
                key,
                min,
                max,
                default,
            } => {
                format!("key {key} int {min}..{max} {default}")
            }
            Self::Boolean { key, default } => {
                format!(
                    "key {key} bool true|false {}",
                    if *default { "true" } else { "false" }
                )
            }
            Self::Choice {
                key,
                choices,
                default,
            } => {
                format!("key {key} enum {} {default}", choices.join("|"))
            }
            Self::Text { key, default } => {
                format!("key {key} string * {default}")
            }
        }
    }
}

pub trait UbgiEngine {
    fn id_name(&self) -> &'static str;
    fn id_version(&self) -> &'static str;
    fn on_ready(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn set(&mut self, _key: &str, _value: &str) -> Result<(), String> {
        Err("unsupported key".to_string())
    }
    fn choose_move(&mut self, game: &Game, dice: Dice) -> Result<String, String>;
}

pub fn run_ubgi_stdio(engine: &mut impl UbgiEngine) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut variant = Variant::Backgammon;
    let mut game = Game::new(variant);
    let mut dice: Option<Dice> = None;
    let variant_enum = VARIANTS
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("|");
    let variant_key = format!("key game.variant enum {variant_enum} backgammon");

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }

        if cmd == "ubgi" {
            reply(&mut stdout, &format!("id name {}", engine.id_name()));
            reply(&mut stdout, "id author bkgm");
            reply(&mut stdout, &format!("id version {}", engine.id_version()));
            reply(&mut stdout, "proto 0.2");
            reply(&mut stdout, &variant_key);
            for spec in engine.options() {
                reply(&mut stdout, &spec.to_key_line());
            }
            reply(&mut stdout, "ubgiok");
            continue;
        }

        if cmd == "isready" {
            match engine.on_ready() {
                Ok(()) => reply(&mut stdout, "readyok"),
                Err(err) => reply(&mut stdout, &format!("error bad_state isready {err}")),
            }
            continue;
        }

        if cmd == "newgame" {
            game = Game::new(variant);
            dice = None;
            continue;
        }

        if cmd == "keys" {
            reply(&mut stdout, &variant_key);
            for spec in engine.options() {
                reply(&mut stdout, &spec.to_key_line());
            }
            continue;
        }

        if let Some(key) = cmd.strip_prefix("get ") {
            let key = key.trim();
            if key == "game.variant" {
                reply(&mut stdout, &format!("value game.variant {}", variant));
            } else if let Some(value) = engine.get(key) {
                reply(&mut stdout, &format!("value {key} {value}"));
            } else {
                reply(&mut stdout, "error unsupported key");
            }
            continue;
        }

        if let Some(rest) = cmd.strip_prefix("set ") {
            let mut it = rest.splitn(2, ' ');
            let key = it.next().unwrap_or("").trim();
            let value = it.next().unwrap_or("").trim();
            if key.is_empty() || value.is_empty() {
                reply(&mut stdout, "error bad_command set");
                continue;
            }
            if key == "game.variant" {
                match value.parse::<Variant>() {
                    Ok(v) => {
                        variant = v;
                        game = Game::new(variant);
                    }
                    Err(_) => reply(&mut stdout, "error bad_value game.variant"),
                }
            } else {
                match engine.set(key, value) {
                    Ok(()) => {}
                    Err(err) if err.starts_with("unsupported") => {
                        reply(&mut stdout, "error unsupported key")
                    }
                    Err(err) if err.starts_with("bad_value") => {
                        reply(&mut stdout, "error bad_value key")
                    }
                    Err(err) => reply(&mut stdout, &format!("error bad_state set {err}")),
                }
            }
            continue;
        }

        if let Some(id) = cmd.strip_prefix("position gnubgid ") {
            match gnuid::decode(variant, id.trim()) {
                Some(pos) => {
                    let _ = game.set_position(pos);
                }
                None => reply(&mut stdout, "error bad_value position"),
            }
            continue;
        }

        if cmd == "position xgid" || cmd.starts_with("position xgid ") {
            reply(&mut stdout, "error unsupported position.xgid");
            continue;
        }

        if let Some(rest) = cmd.strip_prefix("dice ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 2 {
                reply(&mut stdout, "error bad_value dice");
                continue;
            }
            let d1 = parts[0].parse::<usize>();
            let d2 = parts[1].parse::<usize>();
            match (d1, d2) {
                (Ok(a), Ok(b)) if (1..=6).contains(&a) && (1..=6).contains(&b) => {
                    dice = Some(Dice::new(a, b));
                }
                _ => reply(&mut stdout, "error bad_value dice"),
            }
            continue;
        }

        if cmd == "go" || cmd == "go chequer" {
            let Some(current_dice) = dice else {
                reply(&mut stdout, "error bad_state missing.dice");
                continue;
            };
            match engine.choose_move(&game, current_dice) {
                Ok(mv) => reply(&mut stdout, &format!("bestmove {mv}")),
                Err(err) => reply(&mut stdout, &format!("error bad_state move.select {err}")),
            }
            continue;
        }

        if cmd == "quit" {
            break;
        }

        reply(&mut stdout, "error bad_command unknown");
    }
}

fn reply(out: &mut impl Write, line: &str) {
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}
