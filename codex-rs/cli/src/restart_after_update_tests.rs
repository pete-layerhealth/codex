use super::*;
use pretty_assertions::assert_eq;
use std::path::Path;

#[test]
fn relaunch_preserves_arguments_and_inherits_working_directory() -> anyhow::Result<()> {
    let path_dir = tempfile::tempdir()?;
    let executable_name = if cfg!(windows) { "codex.exe" } else { "codex" };
    let executable = path_dir.path().join(executable_name);
    std::fs::write(&executable, "")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(/*mode*/ 0o700))?;
    }

    let args = vec![
        OsString::from("--profile"),
        OsString::from("team profile"),
        OsString::from("prompt with spaces"),
    ];
    let path_env = std::env::join_paths([path_dir.path()])?;
    let command = relaunch_command(&args, &path_env)?;

    assert_eq!(command.get_program(), executable.as_os_str());
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        args.iter().map(OsString::as_os_str).collect::<Vec<_>>()
    );
    assert_eq!(command.get_current_dir(), None);
    assert_eq!(
        command
            .get_envs()
            .find(|(key, _)| *key == RELAUNCHED_AFTER_UPDATE_ENV),
        Some((
            OsStr::new(RELAUNCHED_AFTER_UPDATE_ENV),
            Some(OsStr::new("1"))
        ))
    );
    Ok(())
}

#[test]
fn relaunch_ignores_relative_path_entries() -> anyhow::Result<()> {
    let path_env = std::env::join_paths([Path::new("relative")])?;
    let error = relaunch_command(&[], &path_env).expect_err("relative PATH should be ignored");

    assert!(
        error
            .to_string()
            .contains("could not find the updated `codex` command on PATH")
    );
    Ok(())
}
