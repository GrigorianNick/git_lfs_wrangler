use core::fmt;
use std::process::Command;

pub struct LfsLock {
    pub file: String,
    pub owner: String,
    pub id: u32,
    pub branch: Option<String>
}

impl LfsLock {
    pub fn from_line(line: String) -> Option<LfsLock> {
        let fields: Vec<&str> = line.split_whitespace().filter(|&s| !s.is_empty()).collect();
        match fields.len() {
            3 => Some(LfsLock::new(fields[0].to_string(), fields[1].to_string(), fields[2].to_string(), None)),
            _ => None,
        }
    }

    pub fn new(file: String, owner: String, id: String, branch: Option<String>) -> Self {
        id.strip_prefix("ID:");
        let id_num = match id.trim_start_matches("ID:").parse::<u32>() {
            Ok(val) => val,
            Err(_) => 0,
        };
        LfsLock{
            file: file,
            owner: owner,
            id: id_num,
            branch: branch,
        }
    }

    pub fn unlock(&self) {
        let unlock = ["git lfs unlock -i", &self.id.to_string()].join(" ");
        let _ = Command::new("cmd").args(["/C", &unlock]).output();
    }

    pub fn lock_file(p: String) {
        let lock = ["git lfs lock", &p].join(" ");
        let _ = Command::new("cmd").args(["/C", &lock]).output();
    }
}

impl fmt::Display for LfsLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.branch {
            Some(branch_name) => write!(f, "file: {}; owner: {}; id: {}; branch: {}", self.file, self.owner, self.id, branch_name),
            None => write!(f, "file: {}; owner: {}; id: {}; branch: {}", self.file, self.owner, self.id, "None detected"),
        }
    }
}

pub fn get_locks() -> Vec<LfsLock> {
    let out = Command::new("cmd").args(["/C", "git lfs locks"]).output().expect("Failed to execute process");
    if !out.status.success() {
        return vec![];
    }
    println!("out: {}", String::from_utf8_lossy(&out.stdout));
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    let lines: Vec<&str> = out.split("\n").filter(|&s| !s.is_empty()).collect();
    let locks = lines.iter().map(|&l| LfsLock::from_line(l.to_string()).unwrap()).collect();
    for l in &locks {
        println!("{}", l);
    }
    locks
}