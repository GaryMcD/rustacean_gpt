use anyhow::{anyhow, Error};
use std::process::Command;

pub fn get_token_estimate(text: &str) -> Result<u32, Error> {

    // Define the platform-specific executable paths
    let executable_path = if cfg!(target_os = "windows") {
        "SharpTokenCaller/executables/SharpTokenCaller_win-x64.exe"
    } else if cfg!(target_os = "linux") {
        "SharpTokenCaller/executables/SharpTokenCaller_linux-x64"
    } else if cfg!(target_os = "macos") {
        "SharpTokenCaller/executables/SharpTokenCaller_osx-x64"
    } else {
        panic!("Unsupported platform");
    };

    let output = Command::new(executable_path)
        .arg(text)
        .output()
        .expect("Failed to execute C# project");

    match output.status.success() {
        true => {
            let result_as_string = String::from_utf8_lossy(&output.stdout);
            let result_as_string = result_as_string.trim();
            match result_as_string.parse::<u32>() {
                Ok(value) => Ok(value),
                Err(error) => Err(error.into())
            }
        },
        false => Err(anyhow!("Error running C# project"))
    }
}


#[cfg(test)]
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_estimate_valid() -> Result<()> {
        let input = "This is a valid input.";
        let result = get_token_estimate(input)?;

        assert!(result >= 0, "Token estimate should be non-negative");

        Ok(())
    }
}