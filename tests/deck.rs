use balatro_workshop::{
    deck::{self, Area, Edit, Enhancement, Rank, Seal},
    lua::{self, Key, Value},
};

fn fixture() -> Value {
    lua::parse(include_bytes!("fixtures/deck.lua")).unwrap()
}

fn first_draw_card(data: &Value) -> &Value {
    let Value::Table(cards) = data.get("cardAreas.deck.cards").unwrap() else {
        panic!("Missing draw pile");
    };
    &cards
        .iter()
        .find(|(key, _)| key == &Key::Number("1".into()))
        .unwrap()
        .1
}

#[test]
fn lists_only_owned_playing_cards_in_area_and_numeric_slot_order() {
    let data = fixture();
    let cards = deck::list(&data).unwrap().unwrap();
    assert_eq!(
        cards
            .iter()
            .map(|card| (card.area, card.slot.as_str()))
            .collect::<Vec<_>>(),
        [
            (Area::Deck, "1"),
            (Area::Deck, "10"),
            (Area::Hand, "1"),
            (Area::Discard, "1"),
            (Area::Play, "1")
        ]
    );
    assert_eq!(cards[2].rank, Ok(Rank::Queen));
    assert_eq!(cards[2].enhancement, Ok(Enhancement::Lucky));
    assert_eq!(cards[2].seal, Ok(Seal::Red));
    assert!(deck::list(&lua::parse(b"return {name='Paul'}").unwrap())
        .unwrap()
        .is_none());
}

#[test]
fn rank_updates_sprite_chips_scoring_id_and_sort_data_without_changing_suit_or_position() {
    let mut data = fixture();
    let before = data.clone();
    for (index, (rank, code, chips, face)) in [
        (Rank::Two, "2", "2", "0"),
        (Rank::Three, "3", "3", "0"),
        (Rank::Four, "4", "4", "0"),
        (Rank::Five, "5", "5", "0"),
        (Rank::Six, "6", "6", "0"),
        (Rank::Seven, "7", "7", "0"),
        (Rank::Eight, "8", "8", "0"),
        (Rank::Nine, "9", "9", "0"),
        (Rank::Ten, "T", "10", "0"),
        (Rank::Jack, "J", "10", "0.1"),
        (Rank::Queen, "Q", "10", "0.2"),
        (Rank::King, "K", "10", "0.3"),
        (Rank::Ace, "A", "11", "0.4"),
    ]
    .into_iter()
    .enumerate()
    {
        deck::apply(&mut data, Area::Deck, "1", Edit::Rank(rank)).unwrap();
        let card = first_draw_card(&data);
        assert_eq!(
            card.get("save_fields.card"),
            Some(&Value::String(format!("H_{code}").into_bytes()))
        );
        assert_eq!(
            card.get("base.name"),
            Some(&Value::String(
                format!("{} of Hearts", rank.label()).into_bytes()
            ))
        );
        assert_eq!(
            card.get("base.value"),
            Some(&Value::String(rank.label().as_bytes().to_vec()))
        );
        for (path, value) in [
            ("base.id", (index + 2).to_string()),
            ("base.nominal", chips.into()),
            ("base.face_nominal", face.into()),
        ] {
            assert_eq!(card.get(path), Some(&Value::Number(value)));
        }
    }
    for path in [
        "base.suit",
        "base.suit_nominal",
        "base.suit_nominal_original",
        "base.colour",
        "base.original_value",
        "base.times_played",
        "playing_card",
        "sort_id",
        "rank",
        "edition",
        "ability",
    ] {
        assert_eq!(
            first_draw_card(&before).get(path),
            first_draw_card(&data).get(path),
            "{path}"
        );
    }
    assert_eq!(data.get("cardAreas.hand"), before.get("cardAreas.hand"));
}

#[test]
fn enhancement_transitions_replace_all_effect_fields_and_keep_per_card_state() {
    let mut data = fixture();
    let original = first_draw_card(&data).clone();
    for (target, center, bonus, mult, x_mult, h_x_mult, h_dollars, p_dollars, extra) in [
        (
            Enhancement::Bonus,
            "m_bonus",
            "30",
            "0",
            "1",
            "0",
            "0",
            "0",
            None,
        ),
        (
            Enhancement::Mult,
            "m_mult",
            "0",
            "4",
            "1",
            "0",
            "0",
            "0",
            None,
        ),
        (
            Enhancement::Wild,
            "m_wild",
            "0",
            "0",
            "1",
            "0",
            "0",
            "0",
            None,
        ),
        (
            Enhancement::Glass,
            "m_glass",
            "0",
            "0",
            "2",
            "0",
            "0",
            "0",
            Some("4"),
        ),
        (
            Enhancement::Steel,
            "m_steel",
            "0",
            "0",
            "1",
            "1.5",
            "0",
            "0",
            None,
        ),
        (
            Enhancement::Stone,
            "m_stone",
            "50",
            "0",
            "1",
            "0",
            "0",
            "0",
            None,
        ),
        (
            Enhancement::Gold,
            "m_gold",
            "0",
            "0",
            "1",
            "0",
            "3",
            "0",
            None,
        ),
        (
            Enhancement::Lucky,
            "m_lucky",
            "0",
            "20",
            "1",
            "0",
            "0",
            "20",
            None,
        ),
        (
            Enhancement::Normal,
            "c_base",
            "0",
            "0",
            "1",
            "0",
            "0",
            "0",
            None,
        ),
    ] {
        deck::apply(&mut data, Area::Deck, "1", Edit::Enhancement(target)).unwrap();
        let card = first_draw_card(&data);
        assert_eq!(
            deck::list(&data).unwrap().unwrap()[0].enhancement,
            Ok(target)
        );
        assert_eq!(
            card.get("save_fields.center"),
            Some(&Value::String(center.as_bytes().to_vec()))
        );
        for (path, expected) in [
            ("ability.bonus", bonus),
            ("ability.mult", mult),
            ("ability.x_mult", x_mult),
            ("ability.h_x_mult", h_x_mult),
            ("ability.h_dollars", h_dollars),
            ("ability.p_dollars", p_dollars),
        ] {
            assert_eq!(
                card.get(path),
                Some(&Value::Number(expected.into())),
                "{target:?} {path}"
            );
        }
        assert_eq!(
            card.get("ability.extra"),
            extra.map(|n| Value::Number(n.into())).as_ref()
        );
    }
    let card = first_draw_card(&data);
    assert!(card.get("ability.order").is_none());
    for path in [
        "ability.custom",
        "ability.perma_bonus",
        "ability.played_this_ante",
        "ability.forced_selection",
        "unknown",
        "save_fields.unknown",
        "edition",
        "base",
    ] {
        assert_eq!(card.get(path), original.get(path), "{path}");
    }
}

#[test]
fn seal_edits_round_trip_in_the_hand_and_none_removes_the_field() {
    let mut data = fixture();
    for seal in [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple, Seal::None] {
        deck::apply(&mut data, Area::Hand, "1", Edit::Seal(seal)).unwrap();
        let encoded = balatro_workshop::codec::encode(&data).unwrap();
        data = balatro_workshop::codec::decode(&encoded).unwrap();
        assert_eq!(deck::list(&data).unwrap().unwrap()[2].seal, Ok(seal));
    }
    let Value::Table(cards) = data.get("cardAreas.hand.cards").unwrap() else {
        panic!()
    };
    assert!(cards[0].1.get("seal").is_none());
    assert_eq!(data.get("cardAreas.deck"), fixture().get("cardAreas.deck"));
}

#[test]
fn unsupported_fields_are_preserved_and_other_fields_remain_editable() {
    let mut data = lua::parse(br#"return {cardAreas={deck={cards={
        [1]={playing_card=1, save_fields={center='m_custom',card='C_1'},base={suit='Clubs',value='1'},ability={set='Enhanced',name='Custom'},seal='Custom'},
        [2]={playing_card=2, save_fields={center='m_custom',card='H_4'},base={suit='Hearts',value='4'},ability={set='Enhanced',name='Custom'}}
    }}}}"#).unwrap();
    let before = data.clone();
    for edit in [
        Edit::Rank(Rank::Ace),
        Edit::Enhancement(Enhancement::Gold),
        Edit::Seal(Seal::None),
    ] {
        assert!(deck::apply(&mut data, Area::Deck, "1", edit).is_err());
        assert_eq!(data, before);
    }
    deck::apply(&mut data, Area::Deck, "2", Edit::Rank(Rank::Ten)).unwrap();
    deck::apply(&mut data, Area::Deck, "2", Edit::Seal(Seal::Gold)).unwrap();
    let rows = deck::list(&data).unwrap().unwrap();
    assert_eq!(rows[1].rank, Ok(Rank::Ten));
    assert_eq!(rows[1].seal, Ok(Seal::Gold));
    assert!(rows[1].enhancement.is_err());
}
