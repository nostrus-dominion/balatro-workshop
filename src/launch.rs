//! Ask the desktop's Steam URL handler to launch Balatro without blocking the UI.
use eframe::egui;
use std::{
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

pub const STEAM_URI: &str = "steam://rungameid/2379780";

#[derive(Default)]
pub struct SteamLauncher {
    pending: Option<Receiver<Result<(), String>>>,
}

impl SteamLauncher {
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub fn start(&mut self, ctx: &egui::Context) {
        if self.busy() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.pending = Some(receiver);
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = run_command(steam_command());
            let _ = sender.send(result);
            ctx.request_repaint();
        });
    }

    pub fn poll(&mut self, ctx: &egui::Context) -> Option<Result<(), String>> {
        let receiver = self.pending.as_ref()?;
        let result = match receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(100));
                return None;
            }
            Err(TryRecvError::Disconnected) => {
                Err("The Steam launcher stopped unexpectedly.".into())
            }
        };
        self.pending = None;
        Some(result)
    }
}

fn steam_command() -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut command = Command::new("rundll32.exe");
        command.args(["url.dll,FileProtocolHandler", STEAM_URI]);
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        command
    }
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("/usr/bin/open");
        command.arg(STEAM_URI);
        command
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let mut command = Command::new("xdg-open");
        command.arg(STEAM_URI);
        command
    }
}

fn run_command(mut command: Command) -> Result<(), String> {
    let mut child = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("Could not ask Steam to launch Balatro: {error}. Install Steam and make sure steam:// links open in Steam."))?;
    // Some URL handlers stay alive with Steam. Don't keep the Play button busy
    // for the game's lifetime; reap that process in a detached worker instead.
    for _ in 0..20 {
        match child.try_wait().map_err(|error| error.to_string())? {
            Some(status) if status.success() => return Ok(()),
            Some(status) => return Err(format!("The Steam launch handler failed ({status}). Check that Steam is installed and steam:// links open in Steam.")),
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_request_targets_balatro() {
        let command = steam_command();
        assert!(command.get_args().any(|argument| argument == STEAM_URI));
    }

    #[test]
    fn missing_launch_handler_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        assert!(run_command(Command::new(dir.path().join("missing-launcher"))).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn handler_exit_status_is_checked() {
        let mut success = Command::new("/bin/sh");
        success.args(["-c", "exit 0"]);
        assert!(run_command(success).is_ok());
        let mut failure = Command::new("/bin/sh");
        failure.args(["-c", "exit 1"]);
        assert!(run_command(failure).is_err());
    }
}
