//! Consumer application command and menu definitions.

use eframe::egui;
use std::collections::HashMap;

pub type CommandCallback = Box<dyn FnMut()>;
/// Parse a single-key keyboard shortcut such as Ctrl+E or Ctrl+Shift+S.
/// Unsupported or ambiguous strings are rejected rather than displayed as working shortcuts.
pub fn parse_shortcut(value: &str) -> Option<egui::KeyboardShortcut> {
    let mut modifiers = egui::Modifiers::NONE;
    let mut key = None;
    for token in value.split('+').map(str::trim) {
        let token = token.to_ascii_uppercase();
        if token.is_empty() {
            return None;
        }
        match token.as_str() {
            "CTRL" | "CONTROL" => modifiers.ctrl = true,
            "CMD" | "COMMAND" => modifiers.command = true,
            "ALT" => modifiers.alt = true,
            "SHIFT" => modifiers.shift = true,
            _ => {
                if key.is_some() {
                    return None;
                }
                key = egui::Key::from_name(&token);
                key?;
            }
        }
    }
    let key = key?;
    if modifiers == egui::Modifiers::NONE {
        return None;
    }
    Some(egui::KeyboardShortcut::new(modifiers, key))
}

pub struct CommandDefinition {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub requires_stored_project: bool,
    callback: CommandCallback,
}

impl CommandDefinition {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        callback: impl FnMut() + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            enabled: true,
            requires_stored_project: false,
            callback: Box::new(callback),
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        let shortcut = shortcut.into();
        assert!(
            parse_shortcut(&shortcut).is_some(),
            "invalid command shortcut: {shortcut}"
        );
        self.shortcut = Some(shortcut);
        self
    }

    /// Disable this command until the current project has a confirmed storage location.
    pub fn requires_stored_project(mut self) -> Self {
        self.requires_stored_project = true;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Use a localization key as the command label.
    /// The host resolves it on every menu render, so locale changes need no callback.
    pub fn label_key(mut self, key: impl Into<String>) -> Self {
        self.label = key.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandRegistryError {
    DuplicateId(String),
    NotFound(String),
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, CommandDefinition>,
    order: Vec<String>,
}

impl CommandRegistry {
    /// Dispatch a shortcut only when both the command and its project requirement allow it.
    pub fn dispatch_shortcut_for_project(
        &mut self,
        shortcut: &egui::KeyboardShortcut,
        has_stored_project: bool,
    ) -> Option<String> {
        let id = self
            .order
            .iter()
            .rev()
            .find(|id| {
                self.commands.get(*id).is_some_and(|command| {
                    command.enabled
                        && (!command.requires_stored_project || has_stored_project)
                        && command
                            .shortcut
                            .as_deref()
                            .and_then(parse_shortcut)
                            .as_ref()
                            == Some(shortcut)
                })
            })?
            .clone();
        let _ = self.dispatch(&id);
        Some(id)
    }

    /// Dispatch the most recently registered enabled command matching the shortcut.
    pub fn dispatch_shortcut(&mut self, shortcut: &egui::KeyboardShortcut) -> Option<String> {
        let id = self
            .order
            .iter()
            .rev()
            .find(|id| {
                self.commands.get(*id).is_some_and(|command| {
                    command.enabled
                        && command
                            .shortcut
                            .as_deref()
                            .and_then(parse_shortcut)
                            .as_ref()
                            == Some(shortcut)
                })
            })?
            .clone();
        let _ = self.dispatch(&id);
        Some(id)
    }

    pub fn register(&mut self, command: CommandDefinition) -> Result<(), CommandRegistryError> {
        if self.commands.contains_key(&command.id) {
            return Err(CommandRegistryError::DuplicateId(command.id));
        }
        self.order.push(command.id.clone());
        self.commands.insert(command.id.clone(), command);
        Ok(())
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), CommandRegistryError> {
        let command = self
            .commands
            .get_mut(id)
            .ok_or_else(|| CommandRegistryError::NotFound(id.to_owned()))?;
        command.enabled = enabled;
        Ok(())
    }

    pub fn dispatch(&mut self, id: &str) -> Result<bool, CommandRegistryError> {
        let command = self
            .commands
            .get_mut(id)
            .ok_or_else(|| CommandRegistryError::NotFound(id.to_owned()))?;
        if !command.enabled {
            return Ok(false);
        }
        (command.callback)();
        Ok(true)
    }

    pub fn get(&self, id: &str) -> Option<&CommandDefinition> {
        self.commands.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandDefinition> {
        self.order.iter().filter_map(|id| self.commands.get(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuLocation {
    File,
    Help,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuPlacement {
    Append,
    Before(String),
    After(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItemDefinition {
    pub menu_id: String,
    pub menu_label: String,
    pub location: MenuLocation,
    pub command_id: String,
    pub placement: MenuPlacement,
    pub top_level_placement: MenuPlacement,
    pub access_key: Option<char>,
}

impl MenuItemDefinition {
    /// Alt+letter opens this custom top-level menu.
    pub fn access_key(mut self, key: char) -> Self {
        self.access_key = Some(key.to_ascii_uppercase());
        self
    }

    /// Position the custom top-level menu before a framework menu id.
    pub fn menu_before(mut self, anchor_id: impl Into<String>) -> Self {
        self.top_level_placement = MenuPlacement::Before(anchor_id.into());
        self
    }

    /// Position the custom top-level menu after a framework menu id.
    pub fn menu_after(mut self, anchor_id: impl Into<String>) -> Self {
        self.top_level_placement = MenuPlacement::After(anchor_id.into());
        self
    }

    /// Place this item before a framework menu command id.
    pub fn before(mut self, anchor_id: impl Into<String>) -> Self {
        self.placement = MenuPlacement::Before(anchor_id.into());
        self
    }

    /// Place this item after a framework menu command id.
    pub fn after(mut self, anchor_id: impl Into<String>) -> Self {
        self.placement = MenuPlacement::After(anchor_id.into());
        self
    }

    pub fn file(command_id: impl Into<String>) -> Self {
        Self {
            menu_id: "file".into(),
            menu_label: "File".into(),
            location: MenuLocation::File,
            command_id: command_id.into(),
            placement: MenuPlacement::Append,
            top_level_placement: MenuPlacement::Append,
            access_key: None,
        }
    }
    pub fn help(command_id: impl Into<String>) -> Self {
        Self {
            menu_id: "help".into(),
            menu_label: "Help".into(),
            location: MenuLocation::Help,
            command_id: command_id.into(),
            placement: MenuPlacement::Append,
            top_level_placement: MenuPlacement::Append,
            access_key: None,
        }
    }
    /// Register a custom menu using a localization key for its title.
    pub fn custom_key(
        menu_id: impl Into<String>,
        menu_label_key: impl Into<String>,
        command_id: impl Into<String>,
    ) -> Self {
        Self::custom(menu_id, menu_label_key, command_id)
    }

    pub fn custom(
        menu_id: impl Into<String>,
        menu_label: impl Into<String>,
        command_id: impl Into<String>,
    ) -> Self {
        Self {
            menu_id: menu_id.into(),
            menu_label: menu_label.into(),
            location: MenuLocation::Custom,
            command_id: command_id.into(),
            placement: MenuPlacement::Append,
            top_level_placement: MenuPlacement::Append,
            access_key: None,
        }
    }
}

#[cfg(test)]
mod localized_menu_tests {
    use super::{CommandDefinition, MenuItemDefinition, parse_shortcut};

    #[test]
    fn shortcut_parser_rejects_invalid_values() {
        assert!(parse_shortcut("Ctrl+E").is_some());
        assert!(parse_shortcut("Ctrl+Shift+S").is_some());
        assert!(parse_shortcut("Ctrl+NotAKey").is_none());
        assert!(parse_shortcut("Ctrl+").is_none());
        assert!(parse_shortcut("E").is_none());
    }

    #[test]
    fn duplicate_shortcuts_use_last_enabled_registration() {
        use super::CommandRegistry;
        use std::cell::RefCell;
        use std::rc::Rc;

        let invoked = Rc::new(RefCell::new(Vec::new()));
        let mut registry = CommandRegistry::default();
        for id in ["first", "second", "third"] {
            let log = Rc::clone(&invoked);
            let name = id.to_owned();
            registry
                .register(
                    CommandDefinition::new(id, id, move || {
                        log.borrow_mut().push(name.clone());
                    })
                    .shortcut("Ctrl+E"),
                )
                .unwrap();
        }
        let key = parse_shortcut("Ctrl+E").unwrap();
        assert_eq!(registry.dispatch_shortcut(&key).as_deref(), Some("third"));
        registry.set_enabled("third", false).unwrap();
        assert_eq!(registry.dispatch_shortcut(&key).as_deref(), Some("second"));
        registry.set_enabled("second", false).unwrap();
        assert_eq!(registry.dispatch_shortcut(&key).as_deref(), Some("first"));
        assert_eq!(&*invoked.borrow(), &["third", "second", "first"]);
    }

    #[test]
    fn disabled_shortcuts_are_not_dispatched() {
        use super::CommandRegistry;
        let mut registry = CommandRegistry::default();
        registry
            .register(
                CommandDefinition::new("disabled", "Disabled", || {
                    panic!("disabled callback must not execute");
                })
                .shortcut("Ctrl+E")
                .enabled(false),
            )
            .unwrap();
        let key = parse_shortcut("Ctrl+E").unwrap();
        assert_eq!(registry.dispatch_shortcut(&key), None);
    }

    #[test]
    fn unrelated_shortcuts_do_not_dispatch() {
        use super::CommandRegistry;
        let mut registry = CommandRegistry::default();
        registry
            .register(
                CommandDefinition::new("export", "Export", || {
                    panic!("unrelated shortcut must not execute");
                })
                .shortcut("Ctrl+E"),
            )
            .unwrap();
        assert_eq!(
            registry.dispatch_shortcut(&parse_shortcut("Ctrl+R").unwrap()),
            None
        );
    }

    #[test]
    fn command_label_key_is_stored_without_early_translation() {
        let command =
            CommandDefinition::new("sample.open", "Open", || {}).label_key("sample.menu.open");
        assert_eq!(command.label, "sample.menu.open");
    }

    #[test]
    fn custom_menu_access_key_is_normalized() {
        let item = MenuItemDefinition::custom("tools", "Tools", "app.export").access_key('t');
        assert_eq!(item.access_key, Some('T'));
    }

    #[test]
    fn custom_menu_title_key_is_stored_without_early_translation() {
        let item = MenuItemDefinition::custom_key("sample", "sample.menu.title", "sample.open");
        assert_eq!(item.menu_label, "sample.menu.title");
    }
}

/** Resolve consumer item order relative to stable command IDs.
 * Unknown anchors fall back to the end; cyclic anchors also remain visible.
 * Framework command IDs are supplied as the initial order.
 */
pub fn resolve_menu_order(framework_ids: &[&str], items: &[MenuItemDefinition]) -> Vec<String> {
    let mut order: Vec<String> = framework_ids.iter().map(|id| (*id).to_owned()).collect();
    let mut pending: Vec<&MenuItemDefinition> = items.iter().collect();
    // Iterative resolution supports anchors that refer to other consumer commands.
    while !pending.is_empty() {
        let mut next = Vec::new();
        let mut progress = false;
        for item in pending {
            let anchor = match &item.placement {
                MenuPlacement::Append => None,
                MenuPlacement::Before(id) | MenuPlacement::After(id) => Some(id.as_str()),
            };
            if let Some(anchor) = anchor {
                if let Some(index) = order.iter().position(|id| id == anchor) {
                    let position = if matches!(item.placement, MenuPlacement::After(_)) {
                        index + 1
                    } else {
                        index
                    };
                    order.insert(position, item.command_id.clone());
                    progress = true;
                } else {
                    next.push(item);
                }
            } else {
                order.push(item.command_id.clone());
                progress = true;
            }
        }
        if !progress {
            // Missing and cyclic anchors are appended in registration order.
            order.extend(next.into_iter().map(|item| item.command_id.clone()));
            break;
        }
        pending = next;
    }
    order
}

#[cfg(test)]
mod menu_order_tests {
    use super::{MenuItemDefinition, resolve_menu_order};

    #[test]
    fn positions_before_and_after_framework_commands() {
        let items = vec![
            MenuItemDefinition::file("export").after("save"),
            MenuItemDefinition::file("import").before("save"),
        ];
        assert_eq!(
            resolve_menu_order(&["new", "save"], &items),
            ["new", "import", "save", "export"]
        );
    }

    #[test]
    fn resolves_forward_references_to_consumer_commands() {
        let items = vec![
            MenuItemDefinition::file("a").after("b"),
            MenuItemDefinition::file("b").after("save"),
        ];
        assert_eq!(resolve_menu_order(&["save"], &items), ["save", "b", "a"]);
    }

    #[test]
    fn missing_and_cyclic_anchors_remain_visible() {
        let items = vec![
            MenuItemDefinition::file("a").after("missing"),
            MenuItemDefinition::file("b").after("c"),
            MenuItemDefinition::file("c").after("b"),
        ];
        assert_eq!(
            resolve_menu_order(&["save"], &items),
            ["save", "a", "b", "c"]
        );
    }
}

#[cfg(test)]
mod stored_project_command_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn stored_project_command_is_guarded_even_with_duplicate_shortcut() {
        let ordinary_calls = Rc::new(Cell::new(0));
        let stored_calls = Rc::new(Cell::new(0));
        let mut registry = CommandRegistry::default();
        let calls = ordinary_calls.clone();
        registry
            .register(CommandDefinition::new("ordinary", "Ordinary", move || {
                calls.set(calls.get() + 1);
            }).shortcut("Ctrl+E"))
            .unwrap();
        let calls = stored_calls.clone();
        registry
            .register(CommandDefinition::new("stored", "Stored", move || {
                calls.set(calls.get() + 1);
            }).shortcut("Ctrl+E").requires_stored_project())
            .unwrap();

        let shortcut = parse_shortcut("Ctrl+E").unwrap();
        assert_eq!(registry.dispatch_shortcut_for_project(&shortcut, false), Some("ordinary".into()));
        assert_eq!(ordinary_calls.get(), 1);
        assert_eq!(stored_calls.get(), 0);
        assert_eq!(registry.dispatch_shortcut_for_project(&shortcut, true), Some("stored".into()));
        assert_eq!(stored_calls.get(), 1);
    }

    #[test]
    fn stored_project_command_is_unavailable_without_storage() {
        let mut registry = CommandRegistry::default();
        registry.register(CommandDefinition::new("import", "Import", || {})
            .shortcut("Ctrl+I").requires_stored_project()).unwrap();
        let shortcut = parse_shortcut("Ctrl+I").unwrap();
        assert_eq!(registry.dispatch_shortcut_for_project(&shortcut, false), None);
        assert_eq!(registry.dispatch_shortcut_for_project(&shortcut, true), Some("import".into()));
    }
}
