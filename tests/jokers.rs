use balatro_workshop::{
    codec,
    document::Document,
    jokers::{self, Edition},
    lua::{self, Key, Value},
};

fn fixture() -> Value {
    lua::parse(include_bytes!("fixtures/jokers.lua")).unwrap()
}
fn key(n: &str) -> Key {
    Key::Number(n.into())
}
fn card<'a>(data: &'a Value, n: &str) -> &'a Value {
    let Value::Table(cards) = data.get("cardAreas.jokers.cards").unwrap() else {
        panic!()
    };
    &cards.iter().find(|(k, _)| k == &key(n)).unwrap().1
}
fn number(n: &str) -> Value {
    Value::Number(n.into())
}
fn limit(data: &Value) -> &Value {
    data.get("cardAreas.jokers.config.card_limit").unwrap()
}

#[test]
fn discovers_only_the_six_owned_jokers_in_numeric_order() {
    let data = fixture();
    let owned = jokers::list(&data).unwrap().unwrap();
    assert_eq!(owned.len(), 6);
    assert_eq!(
        owned.iter().map(|j| j.slot.as_str()).collect::<Vec<_>>(),
        ["1", "2", "3", "4", "5", "6"]
    );
    assert_eq!(
        owned.iter().map(|j| j.name.as_str()).collect::<Vec<_>>(),
        [
            "Joker",
            "Abstract Joker",
            "Cavendish",
            "DNA",
            "Joker",
            "Blueprint"
        ]
    );
    assert_eq!(owned[0].edition, Ok(Edition::Foil));
    assert_eq!(owned[5].edition, Ok(Edition::Negative));
    assert_eq!(owned[5].center, "j_blueprint");
}

#[test]
fn handles_profiles_empty_runs_sparse_keys_and_unknown_names() {
    assert!(
        jokers::list(&lua::parse(b"return {career_stats={c_wins=3}}").unwrap())
            .unwrap()
            .is_none()
    );
    assert!(
        jokers::list(&lua::parse(b"return {cardAreas={jokers={cards={}}}}").unwrap())
            .unwrap()
            .unwrap()
            .is_empty()
    );
    let mut data=lua::parse(b"return {cardAreas={jokers={cards={[9]={save_fields={center='j_custom'}}, [2]={ability={name='Known'}}, ['2']={keep=true}},config={card_limit=5}}}}").unwrap();
    let owned = jokers::list(&data).unwrap().unwrap();
    assert_eq!(
        owned.iter().map(|j| j.slot.as_str()).collect::<Vec<_>>(),
        ["2", "9"]
    );
    assert_eq!(owned[1].name, "j_custom");
    jokers::set_edition(&mut data, &key("9"), Edition::Negative).unwrap();
    assert_eq!(limit(&data), &number("6"));
    assert_eq!(
        data.get("cardAreas.jokers.cards.2.keep"),
        Some(&Value::Bool(true))
    );
    assert_eq!(jokers::edition(card(&data, "2")), Ok(Edition::Normal));
}

#[test]
fn all_edition_transitions_adjust_slots_only_for_negative() {
    for old in Edition::ALL {
        for new in Edition::ALL {
            let mut data = fixture();
            jokers::set_edition(&mut data, &key("1"), old).unwrap();
            let before = data.clone();
            let change = jokers::set_edition(&mut data, &key("1"), new).unwrap();
            assert_eq!(
                change,
                i32::from(new == Edition::Negative) - i32::from(old == Edition::Negative)
            );
            assert_eq!(
                limit(&data),
                &number(if new == Edition::Negative { "7" } else { "6" })
            );
            assert_eq!(jokers::edition(card(&data, "1")), Ok(new));
            if old == new {
                assert_eq!(data, before);
            }
            let once = data.clone();
            jokers::set_edition(&mut data, &key("1"), new).unwrap();
            assert_eq!(
                data, once,
                "selecting an edition twice must not add another slot"
            );
            for n in ["2", "3", "4", "5", "6"] {
                assert_eq!(card(&data, n), card(&before, n));
            }
            assert_eq!(data.get("preserved"), before.get("preserved"));
            assert_eq!(
                card(&data, "1").get("custom"),
                card(&before, "1").get("custom")
            );
            assert_eq!(
                card(&data, "1").get("ability"),
                card(&before, "1").get("ability")
            );
        }
    }
}

#[test]
fn writes_complete_bonuses_and_removes_the_previous_edition() {
    let mut data = fixture();
    for (edition, expected) in [
        (
            Edition::Polychrome,
            "return {type='polychrome',polychrome=true,x_mult=1.5}",
        ),
        (
            Edition::Holographic,
            "return {type='holo',holo=true,mult=10}",
        ),
        (Edition::Foil, "return {type='foil',foil=true,chips=50}"),
        (Edition::Negative, "return {type='negative',negative=true}"),
    ] {
        jokers::set_edition(&mut data, &key("1"), edition).unwrap();
        assert_eq!(
            card(&data, "1").get("edition"),
            Some(&lua::parse(expected.as_bytes()).unwrap())
        );
    }
    jokers::set_edition(&mut data, &key("1"), Edition::Normal).unwrap();
    assert!(card(&data, "1").get("edition").is_none());
    assert_eq!(limit(&data), &number("6"));
}

#[test]
fn invalid_slot_counts_leave_the_entire_save_unchanged() {
    for value in [
        number("-1"),
        number("3.5"),
        number("9007199254740991"),
        Value::Bool(true),
    ] {
        let mut data = fixture();
        *data.get_mut("cardAreas.jokers.config.card_limit").unwrap() = value;
        let before = data.clone();
        assert!(jokers::set_edition(&mut data, &key("1"), Edition::Negative).is_err());
        assert_eq!(data, before);
    }
    let mut data = fixture();
    *data.get_mut("cardAreas.jokers.config.card_limit").unwrap() = number("0");
    let before = data.clone();
    assert!(jokers::set_edition(&mut data, &key("6"), Edition::Normal).is_err());
    assert_eq!(data, before);
    // Non-negative transitions do not depend on slot configuration.
    jokers::set_edition(&mut data, &key("1"), Edition::Polychrome).unwrap();
}

#[test]
fn missing_limit_missing_card_and_modded_data_are_not_mutated() {
    for text in [
        "return {cardAreas={jokers={cards={[1]={}}}}}",
        "return {cardAreas={jokers={cards={[1]={}},config={card_limit=5,true_card_limit=6}}}}",
        "return {cardAreas={jokers={cards={[1]={ability={card_limit=1}}},config={card_limit=5}}}}",
        "return {cardAreas={jokers={cards={[1]={edition={key='e_custom',custom=true}}},config={card_limit=5}}}}",
        "return {cardAreas={jokers={cards={[1]={edition={foil=true,negative=true}}},config={card_limit=5}}}}",
        "return {cardAreas={jokers={cards={[1]={edition={type='negative',foil=true}}},config={card_limit=5}}}}",
    ] {
        let mut data=lua::parse(text.as_bytes()).unwrap();
        let before=data.clone();
        assert!(jokers::set_edition(&mut data,&key("1"),Edition::Negative).is_err());
        assert_eq!(data,before);
    }
    let mut data = fixture();
    let before = data.clone();
    assert!(jokers::set_edition(&mut data, &key("99"), Edition::Negative).is_err());
    assert_eq!(data, before);
}

#[test]
fn negative_slot_changes_survive_save_reload_and_revert() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    let original = codec::encode(&fixture()).unwrap();
    std::fs::write(&path, &original).unwrap();
    let mut doc = Document::open(&path).unwrap();
    jokers::set_edition(&mut doc.data, &key("1"), Edition::Negative).unwrap();
    assert!(doc.dirty());
    doc.revert();
    assert_eq!(doc.data, fixture());
    jokers::set_edition(&mut doc.data, &key("1"), Edition::Negative).unwrap();
    let backup = doc.save().unwrap();
    assert_eq!(std::fs::read(backup).unwrap(), original);
    let reopened = Document::open(&path).unwrap();
    assert_eq!(limit(&reopened.data), &number("7"));
    assert_eq!(
        jokers::edition(card(&reopened.data, "1")),
        Ok(Edition::Negative)
    );
    jokers::set_edition(&mut doc.data, &key("6"), Edition::Polychrome).unwrap();
    doc.save().unwrap();
    assert_eq!(limit(&Document::open(&path).unwrap().data), &number("6"));
}
