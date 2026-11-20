//! Edit the base game's playing cards in their saved run areas.
use crate::{
    lua::{Key, Value},
    Result,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Area {
    Deck,
    Hand,
    Discard,
    Play,
}

impl Area {
    const ALL: [Self; 4] = [Self::Deck, Self::Hand, Self::Discard, Self::Play];

    const fn path(self) -> &'static str {
        match self {
            Self::Deck => "cardAreas.deck.cards",
            Self::Hand => "cardAreas.hand.cards",
            Self::Discard => "cardAreas.discard.cards",
            Self::Play => "cardAreas.play.cards",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Deck => "Draw pile",
            Self::Hand => "Hand",
            Self::Discard => "Discard pile",
            Self::Play => "In play",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Self; 13] = [
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
        Self::Ace,
    ];

    pub const fn label(self) -> &'static str {
        [
            "2", "3", "4", "5", "6", "7", "8", "9", "10", "Jack", "Queen", "King", "Ace",
        ][self as usize - 2]
    }

    fn code(self) -> char {
        match self {
            Self::Ten => 'T',
            Self::Jack => 'J',
            Self::Queen => 'Q',
            Self::King => 'K',
            Self::Ace => 'A',
            _ => char::from(b'0' + self as u8),
        }
    }

    fn chips(self) -> u8 {
        match self {
            Self::Ace => 11,
            _ => (self as u8).min(10),
        }
    }

    fn face_nominal(self) -> &'static str {
        match self {
            Self::Jack => "0.1",
            Self::Queen => "0.2",
            Self::King => "0.3",
            Self::Ace => "0.4",
            _ => "0",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit { Spades, Hearts, Clubs, Diamonds }
impl Suit {
    pub const ALL: [Self; 4] = [Self::Spades, Self::Hearts, Self::Clubs, Self::Diamonds];
    pub const fn label(self) -> &'static str {
        match self { Self::Spades => "Spades", Self::Hearts => "Hearts", Self::Clubs => "Clubs", Self::Diamonds => "Diamonds" }
    }
    fn prefix(self) -> char {
        match self { Self::Spades => 'S', Self::Hearts => 'H', Self::Clubs => 'C', Self::Diamonds => 'D' }
    }
    fn nominal(self) -> &'static str {
        match self { Self::Spades => "0.04", Self::Hearts => "0.03", Self::Clubs => "0.02", Self::Diamonds => "0.01" }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enhancement {
    Normal,
    Bonus,
    Mult,
    Wild,
    Glass,
    Steel,
    Stone,
    Gold,
    Lucky,
}

impl Enhancement {
    pub const ALL: [Self; 9] = [
        Self::Normal,
        Self::Bonus,
        Self::Mult,
        Self::Wild,
        Self::Glass,
        Self::Steel,
        Self::Stone,
        Self::Gold,
        Self::Lucky,
    ];

    pub const fn label(self) -> &'static str {
        [
            "Normal", "Bonus", "Mult", "Wild", "Glass", "Steel", "Stone", "Gold", "Lucky",
        ][self as usize]
    }

    fn center(self) -> &'static str {
        [
            "c_base", "m_bonus", "m_mult", "m_wild", "m_glass", "m_steel", "m_stone", "m_gold",
            "m_lucky",
        ][self as usize]
    }

    fn set(self) -> &'static str {
        if self == Self::Normal {
            "Default"
        } else {
            "Enhanced"
        }
    }

    fn name(self) -> String {
        match self {
            Self::Normal => "Default Base".into(),
            Self::Bonus | Self::Mult => self.label().into(),
            _ => self.effect(),
        }
    }

    fn effect(self) -> String {
        if self == Self::Normal {
            "Base".into()
        } else {
            format!("{} Card", self.label())
        }
    }

    fn apply(self, card: &mut Value) {
        let mut label = self.effect();
        if self == Self::Normal {
            label.push_str(" Card");
        }
        put(card, "label", Some(string_value(&label)));
        put(
            card.get_mut("save_fields").unwrap(),
            "center",
            Some(string_value(self.center())),
        );
        let ability = card.get_mut("ability").unwrap();
        put(ability, "name", Some(string_value(&self.name())));
        put(ability, "effect", Some(string_value(&self.effect())));
        put(ability, "set", Some(string_value(self.set())));
        put(
            ability,
            "order",
            (self != Self::Normal).then(|| number(self as u8 + 1)),
        );
        // Replace the previous enhancement's bonuses, leaving per-card state intact.
        for (field, val) in [
            (
                "bonus",
                match self {
                    Self::Bonus => "30",
                    Self::Stone => "50",
                    _ => "0",
                },
            ),
            (
                "mult",
                match self {
                    Self::Mult => "4",
                    Self::Lucky => "20",
                    _ => "0",
                },
            ),
            ("x_mult", if self == Self::Glass { "2" } else { "1" }),
            ("h_x_mult", if self == Self::Steel { "1.5" } else { "0" }),
            ("h_dollars", if self == Self::Gold { "3" } else { "0" }),
            ("p_dollars", if self == Self::Lucky { "20" } else { "0" }),
        ] {
            put(ability, field, Some(number(val)));
        }
        put(ability, "extra", (self == Self::Glass).then(|| number(4)));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seal {
    None,
    Gold,
    Red,
    Blue,
    Purple,
}

impl Seal {
    pub const ALL: [Self; 5] = [Self::None, Self::Gold, Self::Red, Self::Blue, Self::Purple];

    pub const fn label(self) -> &'static str {
        ["None", "Gold", "Red", "Blue", "Purple"][self as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Edit {
    Suit(Suit),
    Rank(Rank),
    Enhancement(Enhancement),
    Seal(Seal),
}

impl Edit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Suit(suit) => suit.label(),
            Self::Rank(rank) => rank.label(),
            Self::Enhancement(enhancement) => enhancement.label(),
            Self::Seal(seal) => seal.label(),
        }
    }
}

#[derive(Debug)]
pub struct Card {
    pub area: Area,
    pub slot: String,
    pub suit: Result<Suit>,
    pub rank: Result<Rank>,
    pub enhancement: Result<Enhancement>,
    pub seal: Result<Seal>,
}

fn number(n: impl ToString) -> Value {
    Value::Number(n.to_string())
}

fn string_value(s: &str) -> Value {
    Value::String(s.as_bytes().to_vec())
}

fn string(value: Option<&Value>) -> Option<&str> {
    match value? {
        Value::String(bytes) => std::str::from_utf8(bytes).ok(),
        _ => None,
    }
}

fn put(table: &mut Value, name: &str, value: Option<Value>) {
    let Value::Table(entries) = table else {
        unreachable!("Card edit validated the table before mutation");
    };
    let index = entries
        .iter()
        .position(|(key, _)| matches!(key, Key::String(bytes) if bytes == name.as_bytes()));
    match (index, value) {
        (Some(index), Some(value)) => entries[index].1 = value,
        (Some(index), None) => {
            entries.remove(index);
        }
        (None, Some(value)) => entries.push((Key::String(name.as_bytes().to_vec()), value)),
        (None, None) => {}
    }
}

fn positive_integer(value: &str) -> Option<f64> {
    value
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite() && *n >= 1.0 && n.fract() == 0.0)
}

fn is_playing_card(card: &Value) -> bool {
    matches!(card.get("playing_card"), Some(Value::Number(n)) if positive_integer(n).is_some())
}

fn suit(card: &Value) -> Result<Suit> {
    Suit::ALL.into_iter().find(|s| string(card.get("base.suit")) == Some(s.label()))
        .ok_or_else(|| "Custom or unrecognized suit. Use Advanced text.".into())
}

fn suit_prefix(card: &Value) -> Option<char> {
    match string(card.get("base.suit"))? {
        "Spades" => Some('S'),
        "Hearts" => Some('H'),
        "Clubs" => Some('C'),
        "Diamonds" => Some('D'),
        _ => None,
    }
}

fn front(prefix: char, rank: Rank) -> String {
    format!("{prefix}_{}", rank.code())
}

fn rank_and_suit(card: &Value) -> Result<(Rank, char)> {
    let values = (|| {
        let suit = suit_prefix(card)?;
        let rank_name = string(card.get("base.value"))?;
        let rank = Rank::ALL
            .into_iter()
            .find(|rank| rank.label() == rank_name)?;
        let saved_front = string(card.get("save_fields.card"))?;
        (front(suit, rank) == saved_front).then_some((rank, suit))
    })();
    values.ok_or_else(|| "Custom or unrecognized rank or suit. Use Advanced text.".into())
}

fn enhancement(card: &Value) -> Result<Enhancement> {
    Enhancement::ALL
        .into_iter()
        .find(|kind| {
            string(card.get("save_fields.center")) == Some(kind.center())
                && string(card.get("ability.set")) == Some(kind.set())
                && string(card.get("ability.name")).is_some_and(|name| name == kind.name())
        })
        .ok_or_else(|| "Custom or unrecognized enhancement. Use Advanced text.".into())
}

fn seal(card: &Value) -> Result<Seal> {
    match card.get("seal") {
        None | Some(Value::Bool(false)) => Ok(Seal::None),
        value => Seal::ALL
            .into_iter()
            .skip(1)
            .find(|seal| string(value) == Some(seal.label()))
            .ok_or_else(|| "Custom or unrecognized seal. Use Advanced text.".into()),
    }
}

/// Return None for saves without playing-card areas. Ignore offers and non-card entries.
pub fn list(data: &Value) -> Result<Option<Vec<Card>>> {
    let mut found_area = false;
    let mut rows = Vec::new();
    for area in Area::ALL {
        let Some(cards) = data.get(area.path()) else {
            continue;
        };
        found_area = true;
        let Value::Table(entries) = cards else {
            return Err(format!(
                "{} has an invalid card list. Inspect it in Advanced text.",
                area.label()
            ));
        };
        let mut indexed = Vec::new();
        for (key, card) in entries {
            let Key::Number(slot) = key else { continue };
            let Some(index) = positive_integer(slot) else {
                continue;
            };
            if is_playing_card(card) {
                indexed.push((
                    index,
                    Card {
                        area,
                        slot: slot.clone(),
                        suit: suit(card),
                        rank: rank_and_suit(card).map(|(rank, _)| rank),
                        enhancement: enhancement(card),
                        seal: seal(card),
                    },
                ));
            }
        }
        indexed.sort_by(|a, b| a.0.total_cmp(&b.0));
        rows.extend(indexed.into_iter().map(|(_, card)| card));
    }
    Ok(found_area.then_some(rows))
}

pub fn apply(data: &mut Value, area: Area, slot: &str, edit: Edit) -> Result<()> {
    let card = match data.get_mut(area.path()) {
        Some(Value::Table(entries)) => entries
            .iter_mut()
            .find_map(|(key, card)| matches!(key, Key::Number(n) if n == slot).then_some(card))
            .filter(|card| positive_integer(slot).is_some() && is_playing_card(card)),
        _ => None,
    }
    .ok_or("This playing card is no longer in the loaded run.")?;

    match edit {
        Edit::Suit(target) => {
            let (rank, _) = rank_and_suit(card)?;
            let current = suit(card)?;
            if current != target {
                put(card.get_mut("save_fields").unwrap(), "card", Some(string_value(&front(target.prefix(), rank))));
                let base = card.get_mut("base").unwrap();
                put(base, "suit", Some(string_value(target.label())));
                put(base, "name", Some(string_value(&format!("{} of {}", rank.label(), target.label()))));
                put(base, "suit_nominal", Some(number(target.nominal())));
                put(base, "suit_nominal_original", Some(number(target.nominal())));
            }
        }
        Edit::Rank(target) => {
            let (current, prefix) = rank_and_suit(card)?;
            if current != target {
                let suit = string(card.get("base.suit")).unwrap();
                let name = format!("{} of {suit}", target.label());
                put(
                    card.get_mut("save_fields").unwrap(),
                    "card",
                    Some(string_value(&front(prefix, target))),
                );
                let base = card.get_mut("base").unwrap();
                put(base, "name", Some(string_value(&name)));
                put(base, "value", Some(string_value(target.label())));
                put(base, "id", Some(number(target as u8)));
                put(base, "nominal", Some(number(target.chips())));
                put(base, "face_nominal", Some(number(target.face_nominal())));
            }
        }
        Edit::Enhancement(target) => {
            if enhancement(card)? != target {
                target.apply(card);
            }
        }
        Edit::Seal(target) => {
            if seal(card)? != target {
                put(
                    card,
                    "seal",
                    (target != Seal::None).then(|| string_value(target.label())),
                );
            }
        }
    }
    Ok(())
}
