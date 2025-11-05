use anyhow::Result;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate};
use std::io;

/// Generate shell completion script for the specified shell.
pub fn completion<T: CommandFactory>(shell: CompletionShell) -> Result<()> {
    let mut cmd = T::command();
    let bin_name = "wok";
    let shell: Shell = shell.into();

    generate(shell, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}

/// Shell types for completion generation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Fish,
    Zsh,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Zsh => Shell::Zsh,
        }
    }
}
