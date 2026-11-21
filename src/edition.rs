//! Shared serialization for base-game card editions.
use crate::{
    lua::{Key, Value},
    Result,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edition {
    Normal,
    Foil,
    Holographic,
    Polychrome,
    Negative,
}

impl Edition {
    pub const ALL: [Self; 5] = [
        Self::Normal,
        Self::Foil,
        Self::Holographic,
        Self::Polychrome,
        Self::Negative,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Foil => "Foil",
            Self::Holographic => "Holographic",
            Self::Polychrome => "Polychrome",
            Self::Negative => "Negative",
        }
    }

    pub fn bonus(self) -> &'static str {
        match self {
            Self::Normal => "No edition bonus",
            Self::Foil => "+50 Chips",
            Self::Holographic => "+10 Mult",
            Self::Polychrome => "×1.5 Mult",
            Self::Negative => "+1 Joker slot",
        }
    }

    fn flag(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Foil => "foil",
            Self::Holographic => "holo",
            Self::Polychrome => "polychrome",
            Self::Negative => "negative",
        }
    }

    pub(crate) fn value(self) -> Option<Value> {
        if self == Self::Normal {
            return None;
        }
        let mut fields = vec![
            (key("type"), Value::String(self.flag().as_bytes().to_vec())),
            (key(self.flag()), Value::Bool(true)),
        ];
        let stat = match self {
            Self::Foil => Some(("chips", "50")),
            Self::Holographic => Some(("mult", "10")),
            Self::Polychrome => Some(("x_mult", "1.5")),
            _ => None,
        };
        if let Some((name, number)) = stat {
            fields.push((key(name), Value::Number(number.into())));
        }
        Some(Value::Table(fields))
    }
}

fn key(name: &str) -> Key {
    Key::String(name.as_bytes().to_vec())
}

pub(crate) fn read(card: &Value) -> Result<Edition> {
    let fields = match card.get("edition") {
        None | Some(Value::Bool(false)) => return Ok(Edition::Normal),
        Some(Value::Table(fields)) if fields.is_empty() => return Ok(Edition::Normal),
        Some(Value::Table(fields)) => fields,
        _ => return Err("Unrecognized edition data. Use Advanced text.".into()),
    };
    let allowed = [
        "type",
        "foil",
        "holo",
        "polychrome",
        "negative",
        "chips",
        "mult",
        "x_mult",
    ];
    if fields.iter().any(
        |(k, _)| !matches!(k, Key::String(k) if allowed.iter().any(|name| name.as_bytes() == k)),
    ) {
        return Err("Custom edition data is preserved. Use Advanced text to edit it.".into());
    }
    let mut selected = None;
    for candidate in Edition::ALL.into_iter().skip(1) {
        match card.get(&format!("edition.{}", candidate.flag())) {
            Some(Value::Bool(true)) if selected.is_none() => selected = Some(candidate),
            None | Some(Value::Bool(false)) => {}
            _ => return Err("Conflicting or invalid edition flags. Use Advanced text.".into()),
        }
    }
    let selected = selected.ok_or("Unrecognized edition. Use Advanced text.")?;
    if let Some(kind) = card.get("edition.type") {
        if kind != &Value::String(selected.flag().as_bytes().to_vec()) {
            return Err("Edition name and flags disagree. Use Advanced text.".into());
        }
    }
    Ok(selected)
}
