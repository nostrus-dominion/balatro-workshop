//! Edition edits for Jokers owned in the loaded run, without rebuilding cards.
use crate::{
    lua::{Key, Value},
    Result,
};

const CARDS: &str = "cardAreas.jokers.cards";
const CONFIG: &str = "cardAreas.jokers.config";

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

    fn value(self) -> Option<Value> {
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

#[derive(Debug)]
pub struct Joker {
    pub key: Key,
    pub slot: String,
    pub name: String,
    pub center: String,
    pub edition: Result<Edition>,
}

fn key(name: &str) -> Key {
    Key::String(name.as_bytes().to_vec())
}

fn string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(bytes)) => String::from_utf8(bytes.clone())
            .ok()
            .filter(|s| !s.is_empty()),
        _ => None,
    }
}

fn table(value: Option<&Value>) -> Option<&Vec<(Key, Value)>> {
    match value {
        Some(Value::Table(entries)) => Some(entries),
        _ => None,
    }
}

fn card<'a>(data: &'a Value, card_key: &Key) -> Result<&'a Value> {
    table(data.get(CARDS))
        .and_then(|cards| cards.iter().find(|(k, _)| k == card_key))
        .map(|(_, value)| value)
        .ok_or_else(|| "This Joker is no longer in the loaded run.".into())
}

/// Return None for files without an owned-Joker area (profiles, settings, etc.).
pub fn list(data: &Value) -> Result<Option<Vec<Joker>>> {
    if data.get("cardAreas.jokers").is_none() {
        return Ok(None);
    }
    let cards = table(data.get(CARDS))
        .ok_or("The saved Joker list is not a table. Inspect it in Advanced text.")?;
    let mut indexed = Vec::new();
    for (key, value) in cards {
        let Key::Number(slot) = key else {
            continue;
        };
        let index = slot.parse::<f64>().map_err(|_| "Invalid Joker index.")?;
        if !index.is_finite() || index < 1.0 || index.fract() != 0.0 {
            continue;
        }
        // Only the owned area is inspected; never shop, booster, deck, or collection cards.
        if string(value.get("ability.set")).is_some_and(|set| set != "Joker") {
            continue;
        }
        let center = string(value.get("save_fields.center")).unwrap_or_default();
        let name = string(value.get("ability.name"))
            .or_else(|| (!center.is_empty()).then(|| center.clone()))
            .unwrap_or_else(|| format!("Joker [{slot}]"));
        indexed.push((
            index,
            Joker {
                key: key.clone(),
                slot: slot.clone(),
                name,
                center,
                edition: edition(value),
            },
        ));
    }
    indexed.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(Some(indexed.into_iter().map(|(_, joker)| joker).collect()))
}

/// Reject unknown edition/slot schemas rather than silently discarding mod data.
pub fn edition(card: &Value) -> Result<Edition> {
    if !matches!(card, Value::Table(_)) {
        return Err("Unrecognized Joker data. Use Advanced text.".into());
    }
    if card.get("ability.card_limit").is_some() || card.get("ability.extra_slots_used").is_some() {
        return Err("Modded Joker slot handling. Use Advanced text.".into());
    }
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

/// Validate both changes before mutating anything. Returns the slot-count delta.
pub fn set_edition(data: &mut Value, card_key: &Key, target: Edition) -> Result<i32> {
    let original = card(data, card_key)?;
    let old = edition(original)?;
    if old == target {
        return Ok(0);
    }
    let delta = i32::from(target == Edition::Negative) - i32::from(old == Edition::Negative);
    let new_limit = if delta != 0 {
        let config = data
            .get(CONFIG)
            .ok_or("Joker slot configuration is missing. No changes made.")?;
        for name in ["true_card_limit", "real_card_limit", "card_limit_ref"] {
            if config.get(name).is_some() {
                return Err(
                    "Modded slot configuration: change this edition in Advanced text.".into(),
                );
            }
        }
        let Some(Value::Number(text)) = config.get("card_limit") else {
            return Err("Joker slot limit is missing or not a number. No changes made.".into());
        };
        let limit = text
            .parse::<f64>()
            .map_err(|_| "Invalid Joker slot limit.")?;
        // Exact integer arithmetic within Lua's double-precision range.
        if !limit.is_finite()
            || !(0.0..=9_007_199_254_740_990.0).contains(&limit)
            || limit.fract() != 0.0
            || limit + f64::from(delta) < 0.0
        {
            return Err("Joker slot limit must be a nonnegative whole number that can be adjusted safely. No changes made.".into());
        }
        Some(Value::Number(format!("{:.0}", limit + f64::from(delta))))
    } else {
        None
    };
    let Value::Table(mut fields) = original.clone() else {
        unreachable!()
    };
    let edition_key = key("edition");
    if let Some(value) = target.value() {
        if let Some((_, existing)) = fields.iter_mut().find(|(k, _)| k == &edition_key) {
            *existing = value;
        } else {
            fields.push((edition_key, value));
        }
    } else {
        fields.retain(|(k, _)| k != &edition_key);
    }
    let Some(Value::Table(cards)) = data.get_mut(CARDS) else {
        unreachable!()
    };
    cards.iter_mut().find(|(k, _)| k == card_key).unwrap().1 = Value::Table(fields);
    if let Some(limit) = new_limit {
        *data.get_mut("cardAreas.jokers.config.card_limit").unwrap() = limit;
    }
    Ok(delta)
}
