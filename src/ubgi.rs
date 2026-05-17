use crate::codecs::move_text::{self, MoveStep};
use crate::dice::Dice;
use crate::Game;

mod runtime;
pub use runtime::run_ubgi_stdio;

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

pub trait UbgiEngine {
    fn id_name(&self) -> &'static str;
    fn id_version(&self) -> &'static str;
    fn on_ready(&mut self) -> UbgiResult<()> {
        Ok(())
    }
    fn options(&self) -> Vec<OptionSpec> {
        Vec::new()
    }
    fn get(&self, _key: &str) -> Option<OptionValue> {
        None
    }
    fn set(&mut self, _key: &str, _value: &OptionValue) -> UbgiResult<()> {
        Err(UbgiError::unsupported("key"))
    }
    fn choose_move(&mut self, game: &Game, dice: Dice) -> UbgiResult<UbgiMove>;
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
