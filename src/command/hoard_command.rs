use serde::{Deserialize, Serialize};

use crate::command::trove::CommandTrove;
use crate::gui::prompts::{prompt_input, prompt_input_validate};

pub trait Parsable {
    fn parse_arguments(matches: &clap::ArgMatches) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoardCommand {
    pub name: String,
    pub namespace: String,
    pub tags: Option<Vec<String>>,
    pub command: String,
    pub description: Option<String>,
}

impl HoardCommand {
    pub fn default() -> Self {
        Self {
            name: "".to_string(),
            namespace: "".to_string(),
            tags: None,
            command: "".to_string(),
            description: None,
        }
    }
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        if self.name.is_empty()
            || self.namespace.is_empty()
            || self.tags.is_none()
            || self.command.is_empty()
            || self.description.is_none()
        {
            return false;
        }
        true
    }

    pub fn tags_as_string(&self) -> String {
        self.tags
            .as_ref()
            .unwrap_or(&vec!["".to_string()])
            .join(",")
    }

    pub fn with_command_string_input(self, default_value: Option<String>) -> Self {
        let command_string: String = prompt_input("Command to hoard", default_value);
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: command_string,
            description: self.description,
        }
    }

    pub fn with_tags_raw(self, tags: &str) -> Self {
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: Some(
                tags.chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            ),
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_tags_input(self, default_value: Option<String>) -> Self {
        let tag_validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("Tags cant contain whitespaces".to_string())
            } else {
                Ok(())
            }
        };
        let tags: String = prompt_input_validate(
            "Give your command some tags ( comma seperated )",
            default_value,
            Some(tag_validator),
        );
        self.with_tags_raw(&tags)
    }

    pub fn with_namespace_input(self, default_namespace: Option<String>) -> Self {
        let namespace: String = prompt_input("Namespace of the command", default_namespace);
        Self {
            name: self.name,
            namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    fn with_name_input_prompt(
        self,
        default_value: Option<String>,
        trove: &CommandTrove,
        prompt_string: &str,
    ) -> Self {
        let namespace = self.namespace.clone();
        let command_names = trove.commands.clone();
        let validator = move |input: &String| -> Result<(), String> {
            if input.contains(' ') {
                Err("The name cant contain whitespaces".to_string())
            } else if command_names
                .iter()
                .filter(|x| x.namespace == namespace)
                .any(|x| x.name == *input)
            {
                Err(
                    "A command with same name exists in the this namespace. Input a different name"
                        .to_string(),
                )
            } else {
                Ok(())
            }
        };
        let name = prompt_input_validate(prompt_string, default_value, Some(validator));
        Self {
            name,
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: self.description,
        }
    }

    pub fn with_name_input(self, default_value: Option<String>, trove: &CommandTrove) -> Self {
        self.with_name_input_prompt(default_value, trove, "Name your command")
    }

    pub fn with_alt_name_input(self, default_value: Option<String>, trove: &CommandTrove) -> Self {
        let name = self.name.clone();
        let command = self.command.clone();
        let namespace = self.namespace.clone();
        self.with_name_input_prompt(
            default_value,
            trove,
            &format!(
                "A command with same name already exists in the namespace '{}'. Enter an alternate name for '{}' with command `{}`",
                namespace,
                name,
                command
            ),
        )
    }

    pub fn with_description_input(self, default_value: Option<String>) -> Self {
        let description_string: String =
            prompt_input("Describe what the command does", default_value);
        Self {
            name: self.name,
            namespace: self.namespace,
            tags: self.tags,
            command: self.command,
            description: Some(description_string),
        }
    }
}

impl Parsable for HoardCommand {
    fn parse_arguments(matches: &clap::ArgMatches) -> Self {
        let mut new_command = Self::default();

        if let Some(n) = matches.value_of("name") {
            new_command.name = n.to_string();
        }
        // Defaults to 'default' namespace
        if let Some(ns) = matches.value_of("namespace") {
            new_command.namespace = ns.to_string();
        }
        // "$ hoard test -t" was run
        // Expects comma seperated tags
        if let Some(tags) = matches.value_of("tags") {
            new_command.tags = Some(
                tags.split(',')
                    .map(std::string::ToString::to_string)
                    .collect(),
            );
        }
        if let Some(c) = matches.value_of("command") {
            new_command.command = c.to_string();
        }
        new_command
    }
}

#[cfg(test)]
mod test_commands {
    use super::*;

    #[test]
    fn one_tag_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = "foo";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn no_tag_as_string() {
        let command = HoardCommand::default();
        let expected = "";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn multiple_tags_as_string() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = "foo,bar";
        assert_eq!(expected, command.tags_as_string());
    }

    #[test]
    fn parse_single_tag() {
        let command = HoardCommand::default().with_tags_raw("foo");
        let expected = Some(vec!["foo".to_string()]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_no_tag() {
        let command = HoardCommand::default();
        let expected = None;
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_multiple_tags() {
        let command = HoardCommand::default().with_tags_raw("foo,bar");
        let expected = Some(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(expected, command.tags);
    }

    #[test]
    fn parse_whitespace_in_tags() {
        let command = HoardCommand::default().with_tags_raw("foo, bar");
        let expected = Some(vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(expected, command.tags);
    }
}
