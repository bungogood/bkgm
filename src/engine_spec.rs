use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineSpecError {
    #[error("invalid engine spec '{0}'")]
    InvalidSpec(String),
    #[error("invalid engine override '{part}' in '{spec}'")]
    InvalidOverride { part: String, spec: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineSpec {
    pub alias: String,
    pub version: Option<String>,
    pub options: BTreeMap<String, String>,
}

pub fn parse_engine_spec(spec: &str) -> Result<EngineSpec, EngineSpecError> {
    let (head, overrides_raw) = match spec.split_once(':') {
        Some((a, b)) => (a.trim(), Some(b.trim())),
        None => (spec.trim(), None),
    };
    let (alias, version) = parse_alias_version(head);
    if alias.is_empty() {
        return Err(EngineSpecError::InvalidSpec(spec.to_string()));
    }
    let mut options = BTreeMap::new();
    if let Some(raw) = overrides_raw {
        if !raw.is_empty() {
            for part in raw.split(',') {
                let p = part.trim();
                if p.is_empty() {
                    continue;
                }
                let (k, v) = p
                    .split_once('=')
                    .ok_or_else(|| EngineSpecError::InvalidOverride {
                        part: p.to_string(),
                        spec: spec.to_string(),
                    })?;
                let key = if k.trim().starts_with("engine.") {
                    k.trim().to_string()
                } else {
                    format!("engine.{}", k.trim())
                };
                options.insert(key, v.trim().to_string());
            }
        }
    }
    Ok(EngineSpec {
        alias: alias.to_string(),
        version,
        options,
    })
}

pub fn format_engine_spec(spec: &EngineSpec) -> String {
    let base = if let Some(v) = &spec.version {
        format!("{}@{}", spec.alias, v)
    } else {
        spec.alias.clone()
    };
    let mut shown = Vec::new();
    if let Some(v) = spec.options.get("engine.ply") {
        shown.push(format!("ply={v}"));
    }
    if let Some(v) = spec.options.get("engine.top_k") {
        shown.push(format!("top_k={v}"));
    }
    for (k, v) in &spec.options {
        if k == "engine.ply" || k == "engine.top_k" {
            continue;
        }
        shown.push(format!("{}={}", k.trim_start_matches("engine."), v));
    }
    if shown.is_empty() {
        base
    } else {
        format!("{base}:{}", shown.join(","))
    }
}

fn parse_alias_version(head: &str) -> (&str, Option<String>) {
    match head.split_once('@') {
        Some((alias, version)) if !version.trim().is_empty() => {
            (alias.trim(), Some(version.trim().to_string()))
        }
        _ => (head.trim(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::{format_engine_spec, parse_engine_spec, EngineSpec};
    use std::collections::BTreeMap;

    #[test]
    fn parse_and_format_roundtrip_basic() {
        let parsed = parse_engine_spec("hawk:ply=2").expect("parse spec");
        assert_eq!(parsed.alias, "hawk");
        assert_eq!(parsed.version, None);
        assert_eq!(parsed.options.get("engine.ply"), Some(&"2".to_string()));
        assert_eq!(format_engine_spec(&parsed), "hawk:ply=2");
    }

    #[test]
    fn parse_and_format_roundtrip_with_version_and_sort() {
        let parsed = parse_engine_spec("hawk@v1.3:top_k=8,ply=2").expect("parse spec");
        assert_eq!(format_engine_spec(&parsed), "hawk@v1.3:ply=2,top_k=8");
    }

    #[test]
    fn format_from_struct() {
        let mut options = BTreeMap::new();
        options.insert("engine.ply".to_string(), "1".to_string());
        let spec = EngineSpec {
            alias: "gnubg".to_string(),
            version: None,
            options,
        };
        assert_eq!(format_engine_spec(&spec), "gnubg:ply=1");
    }
}
