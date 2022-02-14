use crate::interrupt::check_interrupted;
use anyhow::{anyhow, Context, Result};
use std::path::Path;
use std::time::Duration;
use subprocess::{Exec, Popen, Redirection};

/// The result of running a single Cargo command.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CargoResult {
    /// Cargo exited successfully.
    Success,
    /// Cargo failed for some reason.
    Failure,
}

impl CargoResult {
    pub fn success(&self) -> bool {
        matches!(self, CargoResult::Success)
    }
}

/// How frequently to check if cargo finished.
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Run one `cargo` subprocess and with appropriate handling of interrupts.
pub fn run_cargo(cargo_args: &[&str], in_dir: &Path) -> Result<CargoResult> {
    let cargo_bin = "cargo";
    let mut child = Exec::cmd(cargo_bin)
        .stdin(Redirection::None)
        .stdout(Redirection::None)
        .stderr(Redirection::Merge)
        .cwd(in_dir.as_os_str().to_owned())
        .args(cargo_args)
        .env("RUSTFLAGS", "-D warnings -A unused-imports")
        .popen()?;
    let exit_status = loop {
        if let Err(e) = check_interrupted() {
            terminate_child(child)?;
            return Err(e);
        } else if let Some(status) = child.wait_timeout(WAIT_POLL_INTERVAL)? {
            break status;
        }
    };
    if exit_status.success() {
        Ok(CargoResult::Success)
    } else {
        Ok(CargoResult::Failure)
    }
}

#[cfg(unix)]
fn terminate_child(mut child: Popen) -> Result<()> {
    use nix::errno::Errno;
    use nix::sys::signal::{killpg, Signal};

    let pid = nix::unistd::Pid::from_raw(child.pid().expect("child has a pid").try_into().unwrap());
    if let Err(errno) = killpg(pid, Signal::SIGTERM) {
        if errno == Errno::ESRCH {
            // most likely we raced and it's already gone
            return Ok(());
        } else {
            let message = format!("failed to terminate child: {}", errno);
            return Err(anyhow!(message));
        }
    }
    child
        .wait()
        .context("wait for child after terminating pgroup")?;
    Ok(())
}

#[cfg(not(unix))]
fn terminate_child(mut child: Popen) -> Result<()> {
    if let Err(e) = child.terminate() {
        // most likely we raced and it's already gone
        let message = format!("failed to terminate child: {}", e);
        return Err(anyhow!(message));
    }
    child.wait().context("wait for child after kill")?;
    Ok(())
}
