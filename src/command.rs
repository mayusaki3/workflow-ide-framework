//! Consumer application command and menu definitions.

use std::collections::HashMap;

pub type CommandCallback = Box<dyn FnMut()>;

pub struct CommandDefinition {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
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
            callback: Box::new(callback),
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
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
}

impl MenuItemDefinition {
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
        }
    }
    pub fn help(command_id: impl Into<String>) -> Self {
        Self {
            menu_id: "help".into(),
            menu_label: "Help".into(),
            location: MenuLocation::Help,
            command_id: command_id.into(),
            placement: MenuPlacement::Append,
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
        }
    }
}

#[cfg(test)]
mod localized_menu_tests {
    use super::{CommandDefinition, MenuItemDefinition};

    #[test]
    fn command_label_key_is_stored_without_early_translation() {
        let command = CommandDefinition::new("sample.open", "Open", || {})
            .label_key("sample.menu.open");
        assert_eq!(command.label, "sample.menu.open");
    }

    #[test]
    fn custom_menu_title_key_is_stored_without_early_translation() {
        let item = MenuItemDefinition::custom_key(
            "sample", "sample.menu.title", "sample.open",
        );
        assert_eq!(item.menu_label, "sample.menu.title");
    }
}
