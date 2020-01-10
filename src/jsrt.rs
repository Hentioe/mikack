use crate::error::*;
use std::process::{Command, Stdio};

pub fn version() -> Result<String> {
    let mut cmd = Command::new("node");
    cmd.args(&["--version"]).stderr(Stdio::null());
    let output = cmd.output()?;
    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.trim().to_string())
    } else {
        Err(err_msg(
            "Please install Node.js: https://nodejs.org/en/download/",
        ))
    }
}

pub fn eval(code: &str) -> Result<String> {
    let mut cmd = Command::new("node");
    cmd.args(&["-e", code]).stderr(Stdio::null());
    let output = cmd.output()?;
    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.trim().to_string())
    } else {
        Err(err_msg("Javascript execution error"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = version().unwrap();
        assert_eq!(true, version.starts_with("v"));
    }

    #[test]
    fn test_eval() {
        let output = eval("console.log(`hello js`)").unwrap();
        assert_eq!("hello js", output);
    }
}
