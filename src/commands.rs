// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(all(feature = "updater", not(debug_assertions)))]
mod update;

mod backup;
mod restore;
mod update_custom;
mod update_gamedb;
mod verify;

#[cfg(all(feature = "updater", not(debug_assertions)))]
pub use update::Update;

pub use backup::Backup;
pub use restore::Restore;
pub use update_custom::UpdateCustom;
pub use update_gamedb::UpdateGameDb;
pub use verify::Verify;

pub struct Args {
    pub positional: Vec<String>,
    pub flags: Vec<Flag>
}

pub struct Flag {
    pub name: String,
    pub value: Option<String>
}

impl Flag {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned(), value: None }
    }

    pub fn with_value(name: &str, value: String) -> Self {
        Self { name: name.to_owned(), value: Some(value) }
    }
}

impl Args {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut positional = Vec::new();
        let mut flags = Vec::new();

        while let Some(arg) = args.next() {
            if let Some(name) = arg.strip_prefix("--") {
                if let Some(value) = args.next().filter(|next| !next.starts_with('-')) {
                    flags.push(Flag::with_value(name, value));
                } else {
                    flags.push(Flag::new(name));
                }
            } else {
                positional.push(arg);
            }
        }

        Self { positional, flags }
    }

    #[expect(unused, reason = "Will be used in the future")]
    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f.name == name)
    }

    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.iter().find(|f| f.name == name)
    }

    pub fn get_flag_value(&self, name: &str) -> Option<&String> {
        self.get_flag(name)?.value.as_ref()
    }
}

pub trait Command {
    fn run(args: Args, config: &crate::config::Config);
}
