use super::*;

struct Editor {
    app: Workshop,
    ctx: egui::Context,
    output: egui::FullOutput,
    _directory: tempfile::TempDir,
}

impl Editor {
    fn new(lua: &[u8]) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("save.jkr");
        let data = lua::parse(lua).unwrap();
        std::fs::write(&path, balatro_workshop::codec::encode(&data).unwrap()).unwrap();
        let mut editor = Self {
            app: Workshop {
                document: Some(Document::open(&path).unwrap()),
                files: Vec::new(),
                tab: Tab::Quick,
                search: String::new(),
                raw: String::new(),
                raw_base: String::new(),
                status: String::new(),
                error: None,
                pending: None,
                allow_close: false,
                started: Instant::now(),
                smoke: false,
                edits: edits::ScalarEdits::default(),
                document_id: 0,
                launcher: launch::SteamLauncher::default(),
                play_prompt: false,
            },
            ctx: egui::Context::default(),
            output: egui::FullOutput::default(),
            _directory: directory,
        };
        Workshop::configure_style(&editor.ctx);
        editor.app.sync_raw();
        editor.frame(Vec::new());
        editor.frame(Vec::new());
        editor
    }

    fn frame(&mut self, events: Vec<egui::Event>) {
        self.output = self.ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(950.0, 640.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| self.app.show(ctx),
        );
    }

    fn click_text(&mut self, text: &str) {
        self.click_text_match(text, false);
    }

    fn click_text_match(&mut self, text: &str, last: bool) {
        let positions: Vec<_> = self
            .output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::epaint::Shape::Text(shape) = &shape.shape {
                    if shape.galley.text() == text {
                        return Some(shape.pos + shape.galley.size() * 0.5);
                    }
                }
                None
            })
            .collect();
        let position = *(if last {
            positions.last()
        } else {
            positions.first()
        })
        .unwrap_or_else(|| panic!("visible text not found: {text}"));
        self.frame(vec![egui::Event::PointerMoved(position)]);
        for pressed in [true, false] {
            self.frame(vec![egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }]);
        }
    }

    fn key(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        self.frame(vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]);
    }

    fn type_characters(&mut self, text: &str) {
        for character in text.chars() {
            self.frame(vec![egui::Event::Text(character.to_string())]);
            // Separate keypresses with a redraw, like normal human typing.
            self.frame(Vec::new());
        }
    }

    fn value(&self, path: &str) -> &Value {
        self.app.document.as_ref().unwrap().data.get(path).unwrap()
    }
}

#[test]
fn quick_edit_accepts_multiple_digits_across_frames() {
    let mut editor = Editor::new(include_bytes!("../tests/fixtures/run.lua"));
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("987654");
    // Typing stays in memory; no intermediate digit is saved or committed.
    assert!(editor.app.dirty());
    assert_eq!(editor.value("GAME.dollars"), &Value::Number("125".into()));
    let path = editor.app.document.as_ref().unwrap().path.clone();
    assert_eq!(
        Document::open(&path).unwrap().data.get("GAME.dollars"),
        Some(&Value::Number("125".into()))
    );
    // Exercise the same shortcut path as the desktop application.
    editor.key(egui::Key::S, egui::Modifiers::COMMAND);
    assert!(editor.app.error.is_none(), "{:?}", editor.app.error);
    let path = &editor.app.document.as_ref().unwrap().path;
    assert_eq!(
        Document::open(path).unwrap().data.get("GAME.dollars"),
        Some(&Value::Number("987654".into()))
    );
}

#[test]
fn quick_edit_retains_focus_through_incomplete_numbers() {
    let mut editor = Editor::new(include_bytes!("../tests/fixtures/run.lua"));
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.key(egui::Key::Backspace, egui::Modifiers::NONE);
    editor.type_characters("-12.5e-3");
    editor.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert_eq!(
        editor.value("GAME.dollars"),
        &Value::Number("-12.5e-3".into())
    );
}

#[test]
fn quick_edit_retains_text_and_cursor_while_typing() {
    let mut editor = Editor::new(b"return {name='Player', career_stats={c_wins=0}}");
    editor.click_text("Player");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("My profile");
    editor.key(egui::Key::ArrowLeft, egui::Modifiers::NONE);
    editor.key(egui::Key::Backspace, egui::Modifiers::NONE);
    editor.type_characters("L");
    editor.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert_eq!(editor.value("name"), &Value::String(b"My profiLe".to_vec()));
}

#[test]
fn quick_edit_retains_focus_when_visible_rows_change() {
    let mut editor = Editor::new(b"return {GAME={dollars=125, round=83}}");
    editor.click_text("83");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("4");
    // Changing the visible controls must not replace the active text box.
    editor.app.search = "Round".into();
    editor.frame(Vec::new());
    editor.type_characters("56");
    editor.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert_eq!(editor.value("GAME.round"), &Value::Number("456".into()));
}

#[test]
fn quick_edit_commits_on_blur_without_saving() {
    let mut editor = Editor::new(b"return {GAME={dollars=125, round=83}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("123456");
    editor.click_text("83");
    assert_eq!(
        editor.value("GAME.dollars"),
        &Value::Number("123456".into())
    );
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("789");
    editor.key(egui::Key::Enter, egui::Modifiers::NONE);
    assert_eq!(editor.value("GAME.round"), &Value::Number("789".into()));
    let doc = editor.app.document.as_ref().unwrap();
    assert_eq!(
        Document::open(&doc.path).unwrap().data.get("GAME.dollars"),
        Some(&Value::Number("125".into()))
    );
    assert!(doc.last_backup.is_none());
}

#[test]
fn play_prompts_for_unsaved_input_and_cancel_keeps_it() {
    let mut editor = Editor::new(b"return {GAME={dollars=125}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("987");
    editor.click_text("▶  Play Balatro");
    assert!(editor.app.play_prompt);
    assert!(!editor.app.launcher.busy());
    editor.frame(Vec::new());
    editor.click_text("Cancel");
    assert!(!editor.app.play_prompt);
    assert!(editor.app.dirty());
    assert_eq!(editor.value("GAME.dollars"), &Value::Number("987".into()));
    assert!(editor.app.document.as_ref().unwrap().last_backup.is_none());
}

#[test]
fn invalid_entry_cannot_save_or_launch_and_remains_editable() {
    let mut editor = Editor::new(b"return {GAME={dollars=125}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("-");
    let path = editor.app.document.as_ref().unwrap().path.clone();
    let original = std::fs::read(&path).unwrap();
    editor.app.save_and_play(&editor.ctx);
    assert!(editor.app.error.is_some());
    assert!(!editor.app.launcher.busy());
    assert!(editor.app.edits.dirty());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    editor.app.error = None;
    editor.type_characters("123");
    assert!(editor.app.save(false));
    assert_eq!(editor.value("GAME.dollars"), &Value::Number("-123".into()));
}

#[test]
fn switching_to_advanced_text_includes_pending_input() {
    let mut editor = Editor::new(b"return {GAME={dollars=125}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("54321");
    editor.click_text("Advanced text");
    assert!(matches!(editor.app.tab, Tab::Raw));
    assert!(editor.app.raw.contains("54321"));
    assert!(editor.app.document.as_ref().unwrap().last_backup.is_none());
}

#[test]
fn revert_discards_uncommitted_input() {
    let mut editor = Editor::new(b"return {GAME={dollars=125}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("-");
    assert!(editor.app.dirty());
    editor.app.execute(Pending::Revert, &editor.ctx);
    editor.frame(Vec::new());
    assert!(!editor.app.dirty());
    assert_eq!(editor.value("GAME.dollars"), &Value::Number("125".into()));
}

#[test]
fn opening_another_file_clears_previous_input() {
    let mut editor = Editor::new(b"return {GAME={dollars=125}}");
    editor.click_text("125");
    editor.key(egui::Key::A, egui::Modifiers::COMMAND);
    editor.type_characters("999");
    let new_path = editor._directory.path().join("second.jkr");
    let data = lua::parse(b"return {GAME={dollars=42}}").unwrap();
    std::fs::write(&new_path, balatro_workshop::codec::encode(&data).unwrap()).unwrap();
    editor.app.open(new_path);
    editor.frame(Vec::new());
    assert!(!editor.app.dirty());
    assert_eq!(editor.value("GAME.dollars"), &Value::Number("42".into()));
}

#[test]
fn joker_dropdown_changes_foil_to_polychrome_then_negative_and_saves() {
    let mut editor = Editor::new(include_bytes!("../tests/fixtures/jokers.lua"));
    let path = editor.app.document.as_ref().unwrap().path.clone();
    let original = std::fs::read(&path).unwrap();
    editor.click_text("Jokers");
    editor.frame(Vec::new());
    assert!(matches!(editor.app.tab, Tab::Jokers));
    editor.click_text("Foil");
    editor.frame(Vec::new());
    editor.click_text_match("Polychrome", true);
    let owned = jokers::list(&editor.app.document.as_ref().unwrap().data)
        .unwrap()
        .unwrap();
    assert_eq!(owned.len(), 6);
    assert_eq!(owned[0].edition, Ok(jokers::Edition::Polychrome));
    assert_eq!(owned[3].edition, Ok(jokers::Edition::Polychrome));
    assert_eq!(
        editor.value("cardAreas.jokers.config.card_limit"),
        &Value::Number("6".into())
    );
    editor.frame(Vec::new());
    editor.click_text("Polychrome");
    editor.frame(Vec::new());
    editor.click_text_match("Negative", true);
    assert!(editor.app.error.is_none());
    assert_eq!(
        editor.value("cardAreas.jokers.config.card_limit"),
        &Value::Number("7".into())
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    editor.click_text("Advanced text");
    let raw = lua::parse(editor.app.raw.as_bytes()).unwrap();
    assert_eq!(
        jokers::list(&raw).unwrap().unwrap()[0].edition,
        Ok(jokers::Edition::Negative)
    );
    editor.key(egui::Key::S, egui::Modifiers::COMMAND);
    assert!(editor.app.error.is_none(), "{:?}", editor.app.error);
    let doc = editor.app.document.as_ref().unwrap();
    assert_eq!(
        std::fs::read(doc.last_backup.as_ref().unwrap()).unwrap(),
        original
    );
    let saved = Document::open(&path).unwrap();
    assert_eq!(
        saved.data.get("cardAreas.jokers.config.card_limit"),
        Some(&Value::Number("7".into()))
    );
    assert_eq!(
        jokers::list(&saved.data).unwrap().unwrap()[0].edition,
        Ok(jokers::Edition::Negative)
    );
    assert!(!editor.app.dirty());
}

#[test]
fn joker_changes_revert_and_do_not_leak_into_another_file() {
    let mut editor = Editor::new(include_bytes!("../tests/fixtures/jokers.lua"));
    editor.click_text("Jokers");
    editor.frame(Vec::new());
    editor.click_text("Foil");
    editor.frame(Vec::new());
    editor.click_text_match("Negative", true);
    editor.app.execute(Pending::Revert, &editor.ctx);
    editor.frame(Vec::new());
    assert!(!editor.app.dirty());
    assert_eq!(
        editor.value("cardAreas.jokers.config.card_limit"),
        &Value::Number("6".into())
    );
    let rows = jokers::list(&editor.app.document.as_ref().unwrap().data)
        .unwrap()
        .unwrap();
    assert_eq!(rows[0].edition, Ok(jokers::Edition::Foil));
    let new_path = editor._directory.path().join("profile.jkr");
    let data = lua::parse(b"return {career_stats={c_wins=3}}").unwrap();
    std::fs::write(&new_path, balatro_workshop::codec::encode(&data).unwrap()).unwrap();
    editor.app.open(new_path);
    editor.frame(Vec::new());
    editor.click_text("Jokers");
    editor.frame(Vec::new());
    assert!(editor.output.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::Text(t) if t.galley.text()=="Open the current run's save.jkr to see your Jokers.")));
}

#[test]
fn unapplied_raw_edits_block_joker_controls_until_applied() {
    let mut editor = Editor::new(include_bytes!("../tests/fixtures/jokers.lua"));
    editor.click_text("Advanced text");
    editor.app.raw = "return {cardAreas={jokers={config={card_limit=5},cards={}}}}".into();
    editor.click_text("Jokers");
    editor.frame(Vec::new());
    assert!(editor.output.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::Text(t) if t.galley.text().starts_with("Advanced text has unapplied"))));
    editor.click_text("Advanced text");
    editor.frame(Vec::new());
    editor.click_text("Apply text");
    editor.click_text("Jokers");
    editor.frame(Vec::new());
    assert!(jokers::list(&editor.app.document.as_ref().unwrap().data)
        .unwrap()
        .unwrap()
        .is_empty());
}
