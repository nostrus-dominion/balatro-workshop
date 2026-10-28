//! Keep in-progress text separate from save data until the user finishes editing.
use balatro_workshop::lua::{self, Key, Value};
use eframe::egui;
use std::collections::HashMap;

#[derive(Default)]
pub struct ScalarEdits {
    drafts: HashMap<Vec<usize>, Draft>,
}

struct Draft {
    original: Value,
    text: String,
    label: String,
}

impl Draft {
    fn value(&self) -> Result<Value, String> {
        match &self.original {
            Value::Number(_) if lua::valid_number(&self.text) => {
                Ok(Value::Number(self.text.clone()))
            }
            Value::Number(_) => Err(format!("{}: enter a complete, finite number.", self.label)),
            Value::String(_) => Ok(Value::String(self.text.as_bytes().to_vec())),
            _ => unreachable!("Only numbers and UTF-8 strings have text drafts"),
        }
    }

    fn dirty(&self) -> bool {
        match &self.original {
            Value::Number(original) => original != &self.text,
            Value::String(original) => original != self.text.as_bytes(),
            _ => false,
        }
    }
}

impl ScalarEdits {
    pub fn dirty(&self) -> bool {
        self.drafts.values().any(Draft::dirty)
    }

    pub fn clear(&mut self) {
        self.drafts.clear();
    }

    /// Validate every pending edit before changing any save data.
    pub fn commit(&mut self, data: &mut Value) -> Result<(), String> {
        let mut updates = Vec::new();
        for (path, draft) in &self.drafts {
            if draft.dirty() {
                let value = draft.value()?;
                if value_at_mut(data, path).is_none() {
                    return Err(format!(
                        "{}: the edited field no longer exists.",
                        draft.label
                    ));
                }
                updates.push((path.clone(), value));
            }
        }
        for (path, value) in updates {
            *value_at_mut(data, &path).unwrap() = value;
        }
        self.clear();
        Ok(())
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        value: &mut Value,
        path: &[usize],
        label: &str,
        document_id: u64,
    ) {
        match value {
            Value::Number(_) | Value::String(_) => {
                let initial = match value {
                    Value::Number(text) => text.clone(),
                    Value::String(bytes) => match String::from_utf8(bytes.clone()) {
                        Ok(text) => text,
                        Err(_) => {
                            ui.label(format!(
                                "{} binary bytes — edit in Advanced text",
                                bytes.len()
                            ));
                            return;
                        }
                    },
                    _ => unreachable!(),
                };
                let draft = self.drafts.entry(path.to_vec()).or_insert_with(|| Draft {
                    original: value.clone(),
                    text: initial,
                    label: label.into(),
                });
                let response = ui.add(
                    egui::TextEdit::singleline(&mut draft.text)
                        // Independent of widget order, dirty state, and parent UI layout.
                        .id(egui::Id::new(("scalar", document_id, path)))
                        .desired_width(if matches!(value, Value::Number(_)) {
                            200.0
                        } else {
                            230.0
                        }),
                );
                let parsed = draft.value();
                if parsed.is_err() {
                    ui.colored_label(super::RED, "Enter a finite number");
                }
                if response.lost_focus() {
                    if let Ok(updated) = parsed {
                        *value = updated;
                        self.drafts.remove(path);
                    }
                }
            }
            Value::Bool(boolean) => {
                ui.checkbox(boolean, if *boolean { "Enabled" } else { "Disabled" });
            }
            Value::Table(_) => {}
        }
    }
}

pub fn named_path(data: &Value, name: &str) -> Option<Vec<usize>> {
    let mut node = data;
    let mut path = Vec::new();
    for part in name.split('.') {
        let Value::Table(entries) = node else {
            return None;
        };
        let index = entries
            .iter()
            .position(|(key, _)| matches!(key, Key::String(key) if key == part.as_bytes()))?;
        path.push(index);
        node = &entries[index].1;
    }
    Some(path)
}

pub fn value_at_mut<'a>(data: &'a mut Value, path: &[usize]) -> Option<&'a mut Value> {
    let mut node = data;
    for &index in path {
        let Value::Table(entries) = node else {
            return None;
        };
        node = &mut entries.get_mut(index)?.1;
    }
    Some(node)
}
