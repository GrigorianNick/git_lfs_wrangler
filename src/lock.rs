use core::fmt;
use std::{default, iter, process::Command, str::FromStr};
use regex::Regex;
use std::collections::HashMap;

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
        let _ = id.strip_prefix("ID:");
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
        match &self.branch {
            None => (),
            Some(branch) => {
                let tag_name = [self.id.to_string(), branch.to_string()].join("___");
                let unlock_tag = ["git lfs unlock", &tag_name].join(" ");
                let _ = Command::new("cmd").args(["/C", &unlock_tag]).output();
            }
        }
    }

    pub fn lock_file(p: &String) {
        let lock = ["git lfs lock", p].join(" ");
        let _ = Command::new("cmd").args(["/C", &lock]).output();
    }

    fn normalize_path(p: &String) -> String {
        let mut s = p.replace("\\", "/");
        match s.strip_prefix("./") {
            None => s,
            Some(stripped) => stripped.to_string(),
        }
    }

    pub fn lock_file_branch(p: String) {
        let sanitized_p = Self::normalize_path(&p);
        Self::lock_file(&sanitized_p);
        let locks = get_locks();
        for l in locks {
            let sanitized_l = Self::normalize_path(&l.file);
            if sanitized_l == sanitized_p {
                let new_file = [l.id.to_string(), get_branch()].join("___");
                Self::lock_file(&new_file);
                break;
            }
        }
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
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    let lines: Vec<&str> = out.split("\n").filter(|&s| !s.is_empty()).collect();
    let locks: Vec<LfsLock> = lines.iter().map(|&l| LfsLock::from_line(l.to_string()).unwrap()).collect();
    let re = Regex::new(r"[0-9]+___.*").unwrap();

    let mut tag_map = HashMap::<u32, String>::new();
    for lock in &locks {
        if re.is_match(&lock.file) {
            match lock.file.split_once("___") {
                None => (),
                Some((id, branch)) => {
                    tag_map.insert(id.to_string().parse::<u32>().expect("Failed to parse int"), branch.to_string());
                }
            }
        }
    }

    let real_locks = locks.into_iter().filter_map(|mut lock| {
        match tag_map.get(&lock.id) {
            None => {
                if re.is_match(&lock.file) {
                    None
                } else {
                    Some(lock)
                }
            },
            Some(branch) => {
                lock.branch = Some(branch.clone());
                Some(lock)
            }
        }
    }).collect();

    //let _real_locks:Vec<LfsLock> = locks.into_iter().filter(|lock| _branch_tags.contains(&lock.id)).map(|mut lock| {lock.branch = Some("Test".to_string()); lock}).collect();
    for l in &real_locks {
        println!("Real lock:{}", l);
    }
    real_locks
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