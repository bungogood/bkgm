use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::codecs::gnuid;
use crate::codecs::move_text::{self, MoveStep};
use crate::dice::Dice;
use crate::{Game, Variant, VARIANTS};

pub enum OptionSpec {
    Integer {
        key: String,
        min: Option<i64>,
        max: Option<i64>,
        default: i64,
        minor: bool,
    },
    Boolean {
        key: String,
        default: bool,
        minor: bool,
    },
    Choice {
        key: String,
        choices: Vec<String>,
        default: String,
        minor: bool,
    },
    Text {
        key: String,
        default: String,
        minor: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionValue {
    Int(i64),
    Bool(bool),
    Choice(String),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum UbgiError {
    Unsupported(String),
    BadValue(String),
    BadState(String),
}

pub type UbgiResult<T> = Result<T, UbgiError>;
pub type UbgiMove = Vec<MoveStep>;

pub fn parse_ubgi_move(text: &str) -> UbgiResult<UbgiMove> {
    let normalized =
        move_text::normalize(text).ok_or_else(|| UbgiError::bad_value("move.invalid"))?;
    move_text::parse_move_steps(&normalized).ok_or_else(|| UbgiError::bad_value("move.invalid"))
}

impl UbgiError {
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    pub fn bad_value(msg: impl Into<String>) -> Self {
        Self::BadValue(msg.into())
    }

    pub fn bad_state(msg: impl Into<String>) -> Self {
        Self::BadState(msg.into())
    }
}

impl From<String> for UbgiError {
    fn from(value: String) -> Self {
        Self::BadState(value)
    }
}

impl std::fmt::Display for UbgiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(msg) | Self::BadValue(msg) | Self::BadState(msg) => {
                write!(f, "{msg}")
            }
        }
    }
}

impl OptionValue {
    pub fn to_wire(&self) -> String {
        match self {
            Self::Int(v) => v.to_string(),
            Self::Bool(v) => {
                if *v {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::Choice(v) | Self::Text(v) => v.clone(),
        }
    }
}

impl OptionSpec {
    pub fn integer(key: &str, min: i64, max: i64, default: i64) -> Self {
        Self::Integer {
            key: key.to_string(),
            min: Some(min),
            max: Some(max),
            default,
            minor: false,
        }
    }

    pub fn integer_unbounded(key: &str, default: i64) -> Self {
        Self::Integer {
            key: key.to_string(),
            min: None,
            max: None,
            default,
            minor: false,
        }
    }

    pub fn boolean(key: &str, default: bool) -> Self {
        Self::Boolean {
            key: key.to_string(),
            default,
            minor: false,
        }
    }

    pub fn choice(key: &str, choices: &[&str], default: &str) -> Self {
        Self::Choice {
            key: key.to_string(),
            choices: choices.iter().map(|s| (*s).to_string()).collect(),
            default: default.to_string(),
            minor: false,
        }
    }

    pub fn text(key: &str, default: &str) -> Self {
        Self::Text {
            key: key.to_string(),
            default: default.to_string(),
            minor: false,
        }
    }

    pub fn minor(mut self) -> Self {
        match &mut self {
            Self::Integer { minor, .. }
            | Self::Boolean { minor, .. }
            | Self::Choice { minor, .. }
            | Self::Text { minor, .. } => *minor = true,
        }
        self
    }

    pub fn to_key_line(&self) -> String {
        match self {
            Self::Integer {
                key,
                min,
                max,
                default,
                minor,
            } => {
                let base = match (min, max) {
                    (Some(min), Some(max)) => format!("key {key} int {min}..{max} {default}"),
                    _ => format!("key {key} int {default}"),
                };
                if *minor {
                    format!("{base} !")
                } else {
                    base
                }
            }
            Self::Boolean {
                key,
                default,
                minor,
            } => {
                let base = format!("key {key} bool {}", if *default { "true" } else { "false" });
                if *minor {
                    format!("{base} !")
                } else {
                    base
                }
            }
            Self::Choice {
                key,
                choices,
                default,
                minor,
            } => {
                let base = format!("key {key} enum {} {default}", choices.join("|"));
                if *minor {
                    format!("{base} !")
                } else {
                    base
                }
            }
            Self::Text {
                key,
                default,
                minor,
            } => {
                let base = format!("key {key} string * {default}");
                if *minor {
                    format!("{base} !")
                } else {
                    base
                }
            }
        }
    }

    pub fn key(&self) -> &str {
        match self {
            Self::Integer { key, .. }
            | Self::Boolean { key, .. }
            | Self::Choice { key, .. }
            | Self::Text { key, .. } => key,
        }
    }

    pub fn default_value(&self) -> OptionValue {
        match self {
            Self::Integer { default, .. } => OptionValue::Int(*default),
            Self::Boolean { default, .. } => OptionValue::Bool(*default),
            Self::Choice { default, .. } => OptionValue::Choice(default.clone()),
            Self::Text { default, .. } => OptionValue::Text(default.clone()),
        }
    }

    pub fn parse_value(&self, raw: &str) -> Result<OptionValue, String> {
        match self {
            Self::Integer { min, max, .. } => {
                let v: i64 = raw
                    .parse()
                    .map_err(|_| format!("expected int, got '{raw}'"))?;
                if let (Some(min), Some(max)) = (min, max) {
                    if v < *min || v > *max {
                        return Err(format!("must be in range {min}..{max}"));
                    }
                }
                Ok(OptionValue::Int(v))
            }
            Self::Boolean { .. } => match raw {
                "true" => Ok(OptionValue::Bool(true)),
                "false" => Ok(OptionValue::Bool(false)),
                _ => Err(format!("expected bool true|false, got '{raw}'")),
            },
            Self::Choice { choices, .. } => {
                if choices.iter().any(|c| c == raw) {
                    Ok(OptionValue::Choice(raw.to_string()))
                } else {
                    Err(format!("expected one of [{}]", choices.join("|")))
                }
            }
            Self::Text { .. } => Ok(OptionValue::Text(raw.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLineSpec {
    pub name: String,
    pub key_type: String,
    pub domain: Option<String>,
    pub default: String,
    pub minor: bool,
    pub description: String,
}

pub fn parse_key_line(line: &str) -> Option<KeyLineSpec> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "key" {
        return None;
    }
    let name = parts.next()?.to_string();
    let key_type = parts.next()?.to_string();
    let next = parts.next()?.to_string();
    let (domain, default) = match key_type.as_str() {
        "bool" => (None, next),
        "int" => {
            if looks_like_int_domain(&next) {
                (Some(next), parts.next()?.to_string())
            } else {
                (None, next)
            }
        }
        _ => (Some(next), parts.next()?.to_string()),
    };
    let mut minor = false;
    let mut rest: Vec<&str> = parts.collect();
    if rest.first().copied() == Some("!") {
        minor = true;
        rest.remove(0);
    }
    Some(KeyLineSpec {
        name,
        key_type,
        domain,
        default,
        minor,
        description: rest.join(" "),
    })
}

fn looks_like_int_domain(token: &str) -> bool {
    token.contains("..")
}

#[cfg(test)]
mod tests {
    use super::{parse_key_line, OptionSpec, OptionValue};

    #[test]
    fn emits_minor_marker() {
        let line = OptionSpec::integer("engine.threads", 1, 64, 8)
            .minor()
            .to_key_line();
        assert_eq!(line, "key engine.threads int 1..64 8 !");
    }

    #[test]
    fn parses_minor_marker_and_description() {
        let parsed =
            parse_key_line("key engine.seed int 0..999999 42 ! rng seed").expect("parse key");
        assert!(parsed.minor);
        assert_eq!(parsed.name, "engine.seed");
        assert_eq!(parsed.default, "42");
        assert_eq!(parsed.description, "rng seed");
    }

    #[test]
    fn supports_unbounded_int_emit_and_parse() {
        let line = OptionSpec::integer_unbounded("engine.seed", 42).to_key_line();
        assert_eq!(line, "key engine.seed int 42");
        let parsed = parse_key_line(&line).expect("parse key");
        assert_eq!(parsed.key_type, "int");
        assert_eq!(parsed.domain, None);
        assert_eq!(parsed.default, "42");
    }

    #[test]
    fn parses_bool_without_domain() {
        let parsed =
            parse_key_line("key engine.debug bool false ! logging toggles").expect("parse");
        assert_eq!(parsed.key_type, "bool");
        assert_eq!(parsed.domain, None);
        assert_eq!(parsed.default, "false");
        assert!(parsed.minor);
    }

    #[test]
    fn rejects_out_of_range_integer_set_value() {
        let spec = OptionSpec::integer("engine.ply", 1, 4, 2);
        let err = spec.parse_value("7").expect_err("expected range error");
        assert!(err.contains("1..4"));
    }

    #[test]
    fn rejects_invalid_boolean_set_value() {
        let spec = OptionSpec::boolean("engine.debug", false);
        let err = spec.parse_value("yes").expect_err("expected bool error");
        assert!(err.contains("true|false"));
    }

    #[test]
    fn parses_valid_boolean_set_value() {
        let spec = OptionSpec::boolean("engine.debug", false);
        let value = spec.parse_value("true").expect("expected bool true");
        assert_eq!(value, OptionValue::Bool(true));
    }
}

pub trait UbgiEngine {
    fn id_name(&self) -> &'static str;
    fn id_version(&self) -> &'static str;
    fn on_ready(&mut self) -> UbgiResult<()> {
        Ok(())
    }
    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn set(&mut self, _key: &str, _value: &str) -> UbgiResult<()> {
        Err(UbgiError::unsupported("key"))
    }
    fn set_typed(&mut self, key: &str, value: &OptionValue) -> UbgiResult<()> {
        self.set(key, &value.to_wire())
    }
    fn choose_move(&mut self, game: &Game, dice: Dice) -> UbgiResult<UbgiMove>;
}

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
    dice: Option<Dice>,
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
        return vec![format!("value {key} {value}")];
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
    match engine.set_typed(key, &parsed) {
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
            state.dice = Some(Dice::new(a, b));
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
