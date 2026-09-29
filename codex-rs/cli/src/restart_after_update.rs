use anyhow::Context;
use codex_install_context::InstallContext;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::process::Command;

const RELAUNCHED_AFTER_UPDATE_ENV: &str = "CODEX_INTERNAL_RELAUNCHED_AFTER_UPDATE";

pub(crate) fn was_relaunched_after_update() -> bool {
    std::env::var_os(RELAUNCHED_AFTER_UPDATE_ENV).is_some()
}

pub(crate) fn relaunch_codex(args: &[OsString]) -> anyhow::Result<()> {
    let path_env = std::env::var_os("PATH").context("PATH is not set")?;
    let mut command = relaunch_command(args, &path_env)?;
    println!("Relaunching Codex...");
    std::io::Write::flush(&mut std::io::stdout())?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let error = command.exec();
        Err(error).context("failed to relaunch Codex after updating")
    }

    #[cfg(not(unix))]
    {
        let status = command
            .status()
            .context("failed to relaunch Codex after updating")?;
        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        Ok(())
    }
}

fn relaunch_command(args: &[OsString], path_env: &OsStr) -> anyhow::Result<Command> {
    let package_path_dir = InstallContext::current()
        .package_layout
        .as_ref()
        .and_then(|layout| layout.path_dir.as_deref());
    let path_env = std::env::join_paths(
        std::env::split_paths(path_env)
            .filter(|path| path.is_absolute())
            .filter(|path| package_path_dir != Some(path.as_path())),
    )?;
    let executable = which::which_in_global("codex", Some(&path_env))?
        .next()
        .context("could not find the updated `codex` command on PATH")?;
    let mut command = Command::new(executable);
    command.args(args).env(RELAUNCHED_AFTER_UPDATE_ENV, "1");
    Ok(command)
}

#[cfg(test)]
#[path = "restart_after_update_tests.rs"]
mod tests;
