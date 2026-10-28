use crate::lua::Value;

pub fn kind(value: &Value) -> &'static str {
    if value.get("GAME").is_some() {
        "Run"
    } else if value.get("career_stats").is_some() || value.get("progress").is_some() {
        "Profile"
    } else if value.get("unlocked").is_some() {
        "Collection"
    } else if value.get("GAMESPEED").is_some() || value.get("crashreports").is_some() {
        "Settings"
    } else {
        "Save data"
    }
}

// Paths correspond to the upstream save editor at commit 2202fb942c5b.
pub const FIELDS: &[(&str, &str, &str)] = &[
    ("Run", "Money", "GAME.dollars"),
    ("Run", "Ante", "GAME.round_resets.ante"),
    ("Run", "Round", "GAME.round"),
    ("Run", "Win ante", "GAME.win_ante"),
    ("Run", "Hands per round", "GAME.round_resets.hands"),
    ("Run", "Discards per round", "GAME.round_resets.discards"),
    ("Run", "Reroll cost", "GAME.round_resets.reroll_cost"),
    ("Run", "Joker slots", "cardAreas.jokers.config.card_limit"),
    (
        "Run",
        "Consumable slots",
        "cardAreas.consumeables.config.card_limit",
    ),
    ("Run", "Hand size", "cardAreas.hand.config.card_limit"),
    ("Run", "Shop joker slots", "GAME.shop.joker_max"),
    ("Run", "Normal probability", "GAME.probabilities.normal"),
    ("Run", "Bankruptcy threshold", "GAME.bankrupt_at"),
    ("Profile", "Profile name", "name"),
    (
        "Profile",
        "Unlock all (disables achievements)",
        "all_unlocked",
    ),
    (
        "High scores",
        "Highest round",
        "high_scores.furthest_round.amt",
    ),
    (
        "High scores",
        "Highest ante",
        "high_scores.furthest_ante.amt",
    ),
    ("High scores", "Best hand", "high_scores.hand.amt"),
    (
        "High scores",
        "Best win streak",
        "high_scores.win_streak.amt",
    ),
    (
        "High scores",
        "Current streak",
        "high_scores.current_streak.amt",
    ),
    ("High scores", "Collection", "high_scores.collection.amt"),
    ("High scores", "Most money", "high_scores.most_money.amt"),
    ("High scores", "Boss streak", "high_scores.boss_streak.amt"),
    ("Career", "Wins", "career_stats.c_wins"),
    ("Career", "Losses", "career_stats.c_losses"),
    ("Career", "Rounds", "career_stats.c_rounds"),
    ("Career", "Dollars earned", "career_stats.c_dollars_earned"),
    ("Career", "Cards played", "career_stats.c_cards_played"),
    (
        "Career",
        "Cards discarded",
        "career_stats.c_cards_discarded",
    ),
    (
        "Career",
        "Vouchers bought",
        "career_stats.c_vouchers_bought",
    ),
    (
        "Career",
        "Planetarium used",
        "career_stats.c_planetarium_used",
    ),
    ("Career", "Shop rerolls", "career_stats.c_shop_rerolls"),
    ("Career", "Tarots bought", "career_stats.c_tarots_bought"),
    (
        "Career",
        "Shop dollars spent",
        "career_stats.c_shop_dollars_spent",
    ),
    (
        "Career",
        "Face cards played",
        "career_stats.c_face_cards_played",
    ),
    ("Career", "Planets bought", "career_stats.c_planets_bought"),
    ("Career", "Hands played", "career_stats.c_hands_played"),
    (
        "Career",
        "Tarot reading used",
        "career_stats.c_tarot_reading_used",
    ),
    ("Career", "Jokers sold", "career_stats.c_jokers_sold"),
    (
        "Career",
        "Playing cards bought",
        "career_stats.c_playing_cards_bought",
    ),
    ("Career", "Cards sold", "career_stats.c_cards_sold"),
    (
        "Career",
        "Single-hand round streak",
        "career_stats.c_single_hand_round_streak",
    ),
    (
        "Career",
        "Interest cap streak",
        "career_stats.c_round_interest_cap_streak",
    ),
    ("Settings", "Active profile", "profile"),
    ("Settings", "Crash reports", "crashreports"),
    ("Settings", "Tutorial complete", "tutorial_complete"),
    ("Settings", "Game speed", "GAMESPEED"),
    ("Settings", "Language", "language"),
];
