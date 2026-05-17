use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::codecs::gnuid;
use crate::codecs::move_text;
use crate::{Game, Variant, VARIANTS};

use super::{OptionSpec, OptionValue, UbgiEngine, UbgiError};

enum ProtocolError {
    BadCommand(String),
    BadValue(String),
    BadState(String),
    Unsupported(String),
}

impl ProtocolError {
    fn to_line(&self) -> String {
        match self {
            Self::BadCommand(detail) => format!("error bad_command {detail}"),
            Self::BadValue(detail) => format!("error bad_value {detail}"),
            Self::BadState(detail) => format!("error bad_state {detail}"),
            Self::Unsupported(detail) => format!("error unsupported {detail}"),
        }
    }
}

enum Command {
    Ubgi,
    IsReady,
    NewGame,
    Keys,
    Get(String),
    Set { key: String, value: String },
    PositionGnubgid(String),
    PositionXgid,
    PositionUnknown,
    Dice(String),
    GoChequer,
    GoCube,
    GoTurn,
    Quit,
    Unknown,
}

struct SessionState {
    variant: Variant,
    game: Game,
    dice: Option<crate::Dice>,
    option_specs: Vec<OptionSpec>,
    option_values: HashMap<String, OptionValue>,
    variant_key: String,
}

impl SessionState {
    fn new(engine: &impl UbgiEngine) -> Self {
        let option_specs = engine.options();
        let option_values = option_specs
            .iter()
            .map(|spec| (spec.key().to_string(), spec.default_value()))
            .collect();
        let variant = Variant::Backgammon;
        let variant_enum = VARIANTS
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("|");
        let variant_key = format!("key game.variant enum {variant_enum} backgammon");
        Self {
            variant,
            game: Game::new(variant),
            dice: None,
            option_specs,
            option_values,
            variant_key,
        }
    }
}

pub fn run_ubgi_stdio(engine: &mut impl UbgiEngine) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = SessionState::new(engine);

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }
        let command = parse_command(cmd);
        let should_quit = matches!(command, Command::Quit);
        for line in handle_command(engine, &mut state, command) {
            reply(&mut stdout, &line);
        }
        if should_quit {
            break;
        }
    }
}

fn parse_command(line: &str) -> Command {
    if line == "ubgi" {
        return Command::Ubgi;
    }
    if line == "isready" {
        return Command::IsReady;
    }
    if line == "newgame" {
        return Command::NewGame;
    }
    if line == "keys" {
        return Command::Keys;
    }
    if let Some(key) = line.strip_prefix("get ") {
        return Command::Get(key.trim().to_string());
    }
    if let Some(rest) = line.strip_prefix("set ") {
        let mut it = rest.splitn(2, ' ');
        let key = it.next().unwrap_or("").trim().to_string();
        let value = it.next().unwrap_or("").trim().to_string();
        return Command::Set { key, value };
    }
    if let Some(id) = line.strip_prefix("position gnubgid ") {
        return Command::PositionGnubgid(id.trim().to_string());
    }
    if line == "position xgid" || line.starts_with("position xgid ") {
        return Command::PositionXgid;
    }
    if line.starts_with("position ") {
        return Command::PositionUnknown;
    }
    if let Some(rest) = line.strip_prefix("dice ") {
        return Command::Dice(rest.to_string());
    }
    if line == "go" || line == "go chequer" {
        return Command::GoChequer;
    }
    if line == "go cube" {
        return Command::GoCube;
    }
    if line == "go turn" {
        return Command::GoTurn;
    }
    if line == "quit" {
        return Command::Quit;
    }
    Command::Unknown
}

fn handle_command(
    engine: &mut impl UbgiEngine,
    state: &mut SessionState,
    command: Command,
) -> Vec<String> {
    match command {
        Command::Ubgi => {
            let mut out = vec![
                format!("id name {}", engine.id_name()),
                "id author bkgm".to_string(),
                format!("id version {}", engine.id_version()),
                "proto 0.2".to_string(),
            ];
            out.extend(render_key_lines(state));
            out.push("ubgiok".to_string());
            out
        }
        Command::IsReady => match engine.on_ready() {
            Ok(()) => vec!["readyok".to_string()],
            Err(err) => vec![ProtocolError::BadState(format!("isready {err}")).to_line()],
        },
        Command::NewGame => {
            state.game = Game::new(state.variant);
            state.dice = None;
            Vec::new()
        }
        Command::Keys => render_key_lines(state),
        Command::Get(key) => handle_get(engine, state, &key),
        Command::Set { key, value } => handle_set(engine, state, &key, &value),
        Command::PositionGnubgid(id) => {
            if let Some(pos) = gnuid::decode(state.variant, id.trim()) {
                let _ = state.game.set_position(pos);
                Vec::new()
            } else {
                vec![ProtocolError::BadValue("position".to_string()).to_line()]
            }
        }
        Command::PositionXgid => {
            vec![ProtocolError::Unsupported("position.xgid".to_string()).to_line()]
        }
        Command::PositionUnknown => vec![ProtocolError::BadCommand(
            "expected: position gnubgid <GNU_POSITION_ID>".to_string(),
        )
        .to_line()],
        Command::Dice(rest) => handle_dice(state, &rest),
        Command::GoChequer => handle_go(engine, state),
        Command::GoCube => vec![ProtocolError::Unsupported("go cube".to_string()).to_line()],
        Command::GoTurn => vec![ProtocolError::Unsupported("go turn".to_string()).to_line()],
        Command::Quit => Vec::new(),
        Command::Unknown => vec![ProtocolError::BadCommand("unknown".to_string()).to_line()],
    }
}

fn render_key_lines(state: &SessionState) -> Vec<String> {
    let mut out = vec![state.variant_key.clone()];
    for spec in &state.option_specs {
        out.push(spec.to_key_line());
    }
    out
}

fn handle_get(engine: &impl UbgiEngine, state: &SessionState, key: &str) -> Vec<String> {
    if key == "game.variant" {
        return vec![format!("value game.variant {}", state.variant)];
    }
    if let Some(value) = state.option_values.get(key) {
        return vec![format!("value {key} {}", value.to_wire())];
    }
    if let Some(value) = engine.get(key) {
        return vec![format!("value {key} {}", value.to_wire())];
    }
    vec![ProtocolError::Unsupported("key".to_string()).to_line()]
}

fn handle_set(
    engine: &mut impl UbgiEngine,
    state: &mut SessionState,
    key: &str,
    value: &str,
) -> Vec<String> {
    if key.is_empty() || value.is_empty() {
        return vec![
            ProtocolError::BadCommand("expected: set <key> <value>".to_string()).to_line(),
        ];
    }
    if key == "game.variant" {
        return match value.parse::<Variant>() {
            Ok(v) => {
                state.variant = v;
                state.game = Game::new(v);
                Vec::new()
            }
            Err(_) => vec![ProtocolError::BadValue("game.variant".to_string()).to_line()],
        };
    }
    let Some(spec) = state.option_specs.iter().find(|spec| spec.key() == key) else {
        return vec![ProtocolError::Unsupported("key".to_string()).to_line()];
    };
    let parsed = match spec.parse_value(value) {
        Ok(v) => v,
        Err(_) => return vec![ProtocolError::BadValue(key.to_string()).to_line()],
    };
    match engine.set(key, &parsed) {
        Ok(()) => {
            state.option_values.insert(key.to_string(), parsed);
            Vec::new()
        }
        Err(err) => vec![map_engine_set_error(err, key).to_line()],
    }
}

fn map_engine_set_error(err: UbgiError, key: &str) -> ProtocolError {
    match err {
        UbgiError::Unsupported(_) => ProtocolError::Unsupported("key".to_string()),
        UbgiError::BadValue(detail) => {
            if detail.is_empty() {
                ProtocolError::BadValue(key.to_string())
            } else {
                ProtocolError::BadValue(detail)
            }
        }
        UbgiError::BadState(detail) => ProtocolError::BadState(format!("set {detail}")),
    }
}

fn handle_dice(state: &mut SessionState, rest: &str) -> Vec<String> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() != 2 {
        return vec![ProtocolError::BadValue("dice".to_string()).to_line()];
    }
    let d1 = parts[0].parse::<usize>();
    let d2 = parts[1].parse::<usize>();
    match (d1, d2) {
        (Ok(a), Ok(b)) if (1..=6).contains(&a) && (1..=6).contains(&b) => {
            state.dice = Some(crate::Dice::new(a, b));
            Vec::new()
        }
        _ => vec![ProtocolError::BadValue("dice".to_string()).to_line()],
    }
}

fn handle_go(engine: &mut impl UbgiEngine, state: &SessionState) -> Vec<String> {
    let Some(current_dice) = state.dice else {
        return vec![ProtocolError::BadState("missing.dice".to_string()).to_line()];
    };
    match engine.choose_move(&state.game, current_dice) {
        Ok(mv) => vec![format!("bestmove {}", move_text::format_move_steps(&mv))],
        Err(err) => vec![ProtocolError::BadState(format!("move.select {err}")).to_line()],
    }
}

fn reply(out: &mut impl Write, line: &str) {
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}
