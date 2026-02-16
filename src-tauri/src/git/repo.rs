use std::process::Command;

/// Register weaver as a git mergetool in the user's global gitconfig.
pub fn register_mergetool() -> Result<String, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    let exe_path = exe.to_string_lossy();

    let cmd_value = format!("{} \"$BASE\" \"$LOCAL\" \"$REMOTE\" \"$MERGED\"", exe_path);

    Command::new("git")
        .args(["config", "--global", "mergetool.weaver.cmd", &cmd_value])
        .status()
        .map_err(|e| format!("Failed to set mergetool.weaver.cmd: {}", e))?;

    Command::new("git")
        .args(["config", "--global", "mergetool.weaver.trustExitCode", "true"])
        .status()
        .map_err(|e| format!("Failed to set trustExitCode: {}", e))?;

    Ok(format!("Registered weaver as git mergetool.\nCommand: {}", cmd_value))
}
