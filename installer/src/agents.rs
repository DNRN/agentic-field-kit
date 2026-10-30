//! Handing a new project to the agent (ADR 0017): the agents the installer
//! knows how to start with an opening prompt, and starting one. Each entry is
//! checked against that tool's documentation or `--help`; an agent that isn't
//! here gets the printed next step instead.

use std::io;
use std::path::Path;
use std::process::Command;

use crate::scope::on_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Agent {
    pub name: &'static str,
    /// The command on PATH. The prompt is its one argument.
    pub command: &'static str,
    /// When and how the argument form was checked.
    pub checked: &'static str,
}

pub const AGENTS: &[Agent] = &[
    Agent {
        name: "Claude Code",
        command: "claude",
        checked: "2026-10-30, claude --help: `claude [prompt]` starts an interactive session",
    },
    Agent {
        name: "omp",
        command: "omp",
        checked: "2026-10-30, omp --help: `omp \"<prompt>\"` is interactive mode with an initial prompt",
    },
];

/// The opening prompt: the folder is the working directory.
pub const PROMPT: &str = "Start a new project in this folder with the project-init skill.";

/// The table's agents whose command is on PATH.
pub fn available() -> Vec<&'static Agent> {
    AGENTS.iter().filter(|a| on_path(a.command)).collect()
}

/// The agent's command: the prompt as one argument, no shell, run in `dir`.
pub fn command(program: &str, dir: &Path) -> Command {
    let mut cmd = Command::new(program);
    cmd.arg(PROMPT).current_dir(dir);
    cmd
}

/// Starts the agent in `dir`. On Unix the installer becomes the agent, so
/// this returns only on failure; on Windows it waits and returns the agent's
/// exit status.
pub fn launch(agent: &Agent, dir: &Path) -> io::Result<i32> {
    let mut cmd = command(agent.command, dir);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(cmd.exec())
    }
    #[cfg(not(unix))]
    {
        cmd.status().map(|s| s.code().unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_records_its_check() {
        for a in AGENTS {
            assert!(a.checked.starts_with("20"), "{}: {}", a.name, a.checked);
            assert!(!a.command.contains(' '), "{}", a.command);
        }
    }

    #[test]
    fn the_prompt_is_one_argument_and_the_folder_is_the_working_directory() {
        let dir = Path::new("/tmp/my 'odd' \"project\"");
        let cmd = command("claude", dir);
        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args, [PROMPT]);
        assert_eq!(cmd.get_current_dir(), Some(dir));
        assert_eq!(cmd.get_program(), "claude");
    }

    #[cfg(unix)]
    #[test]
    fn a_started_agent_sees_the_folder_and_the_prompt() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("new project");
        std::fs::create_dir_all(&dir).unwrap();
        let script = tmp.path().join("fake-agent");
        std::fs::write(&script, "#!/bin/sh\npwd\nprintf '%s|' \"$@\"\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let out = command(script.to_str().unwrap(), &dir).output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        let real = dunce::canonicalize(&dir).unwrap();
        assert_eq!(text, format!("{}\n{PROMPT}|", real.display()));
    }
}
