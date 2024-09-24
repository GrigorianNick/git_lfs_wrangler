use std::process::Command;

pub fn get_user() -> String {
    let out = Command::new("cmd").args(["/C", "git config --get user.name"]).output();
    match out {
        Err(_) => "".to_string(),
        Ok(output) => {
            let mut s = String::from_utf8_lossy(&output.stdout).to_string();
            if s.ends_with('\n') {
                s.pop();
            }
            s
        }
    }
}

pub fn get_branch() -> String {
    let out = Command::new("cmd").args(["/C", "git branch --show-current"]).output();
    match out {
        Err(_) => "".to_string(),
        Ok(output) => {
            let mut s = String::from_utf8_lossy(&output.stdout).to_string();
            if s.ends_with('\n') {
                s.pop();
            }
            s
        }
    }
}