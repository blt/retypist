use crate::interrupt::check_interrupted;
use anyhow::{anyhow, Context, Result};
use std::borrow::Cow;
use std::env;
use std::path::Path;
use std::time::Duration;
use subprocess::{Popen, PopenConfig, Redirection};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GitResult {
    /// Cargo exited successfully.
    Success,
    /// Cargo failed for some reason.
    Failure,
}

impl GitResult {
    pub fn success(&self) -> bool {
        matches!(self, GitResult::Success)
    }
}

/// How frequently to check if cargo finished.
const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Run one `cargo` subprocess and with appropriate handling of interrupts.
pub fn run_git(git_args: &[&str], in_dir: &Path) -> Result<GitResult> {
    let git_bin: Cow<str> = env::var("GIT")
        .map(Cow::from)
        .unwrap_or(Cow::Borrowed("git"));

    let mut argv: Vec<&str> = vec![&git_bin];
    argv.extend(git_args.iter());
    let mut child = Popen::create(
        &argv,
        PopenConfig {
            stdin: Redirection::None,
            stdout: Redirection::None,
            stderr: Redirection::Merge,
            cwd: Some(in_dir.as_os_str().to_owned()),
            ..setpgid_on_unix()
        },
    )
    .with_context(|| format!("failed to spawn {} {}", git_bin, git_args.join(" ")))?;
    let exit_status = loop {
        if let Err(e) = check_interrupted() {
            terminate_child(child)?;
            return Err(e);
        } else if let Some(status) = child.wait_timeout(WAIT_POLL_INTERVAL)? {
            break status;
        }
    };
    if exit_status.success() {
        Ok(GitResult::Success)
    } else {
        Ok(GitResult::Failure)
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

#[cfg(unix)]
fn setpgid_on_unix() -> PopenConfig {
    PopenConfig {
        setpgid: true,
        ..Default::default()
    }
}

#[cfg(not(unix))]
fn setpgid_on_unix() -> PopenConfig {
    Default::default()
}
