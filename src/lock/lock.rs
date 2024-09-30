use core::fmt;
use std::{default, iter, process::Command, str::FromStr};
use regex::Regex;
use std::collections::HashMap;
use crate::git;
use crate::lock::tag::tag::Tag;

use super::tag;

pub struct LfsLock {
    pub file: String,
    pub owner: String,
    pub id: u32,
    pub branch: Option<String>,
    pub dir: Option<String>,
    pub queue: Vec<u32>,
    pub tags: Vec<Box<dyn Tag>>,
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
            dir: None,
            queue: vec![],
            tags: vec![],
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

    pub fn lock_file(p: &String) -> Option<LfsLock>{
        let lock = ["git lfs lock", p].join(" ");
        let _ = Command::new("cmd").args(["/C", &lock]).output();
        let locks = get_locks().into_iter().filter(|l| {
            println!("l.file: {} :--: {}", l.file, p);
         Self::normalize_path(&l.file) == Self::normalize_path(p)});
        locks.last()
    }

    fn normalize_path(p: &String) -> String {
        let mut s = p.replace("\\", "/");
        match s.strip_prefix("./") {
            None => s,
            Some(stripped) => stripped.to_string(),
        }
    }

    pub fn enqueue(&self) {
        let queue_prefix = match self.queue.last()
        {
            None =>  ["Q", &self.id.to_string()].join(""),
            Some(id) => ["Q", &id.to_string()].join(""),
        };
        //let queue_prefix = ["Q", &self.id.to_string()].join("");
        let queue_tag = [queue_prefix.to_string(), self.file.to_string()].join("___");
        LfsLock::lock_file(&queue_tag);
    }
}

impl fmt::Display for LfsLock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.branch {
            Some(branch_name) => write!(f, "file: {}; owner: {}; id: {}; branch: {}; queue: {:?}", self.file, self.owner, self.id, branch_name, self.queue),
            None => write!(f, "file: {}; owner: {}; id: {}; branch: {}; queue: {:?}", self.file, self.owner, self.id, "None detected", self.queue),
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


    let mut lock_map = HashMap::<u32, LfsLock>::new();
    let mut tags = Vec::<Box<dyn tag::Tag>>::new();
    for lock in locks {
        match tag::get_tag(&lock) {
            None => {
                lock_map.insert(lock.id, lock);
            }
            Some(tag) => {
                tags.push(tag);
            },
        }
    }
    for tag in tags {
        match lock_map.get_mut(&tag.get_target_id()) {
            None => (),
            Some(lock) => {
                tag.apply(lock);
            },
        }
    }
    lock_map.into_iter().map(|(_id, lock)| lock).collect()
    //vec![]

    /*let re = Regex::new(r"^[0-9]+___.*").unwrap();

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

    // lock.id -> next in line
    let mut queue_map = HashMap::<u32, u32>::new();
    let queue_re = Regex::new(r"Q(?<id>[0-9]+)___.*").unwrap();
    for lock in &locks {
        match queue_re.captures(&lock.file) {
            None => {
                println!("Failed to find in: {}", lock.file);
            },
            Some(c) => {
                println!("inserting: {}", &c["id"]);
                queue_map.insert(c["id"].parse::<u32>().expect("Failed to parse int"), lock.id);
            }
        }
    }

    let real_locks = locks.into_iter().filter_map(|mut lock| {
        if lock.file.contains("___") {
            return None
        }
        match tag_map.get(&lock.id) {
            None => (),
            Some(branch) => {
                lock.branch = Some(branch.clone());
            }
        }
        let mut queue_id = lock.id;
        let mut queue = vec![];
        while let Some(next_id) = queue_map.get(&queue_id) {
            queue.push(*next_id);
            queue_id = *next_id;
        }
        println!("queue: {:?}", queue);
        println!("map: {:?}", queue_map);
        lock.queue = queue;
        Some(lock)
    }).collect();

    //let _real_locks:Vec<LfsLock> = locks.into_iter().filter(|lock| _branch_tags.contains(&lock.id)).map(|mut lock| {lock.branch = Some("Test".to_string()); lock}).collect();
    for l in &real_locks {
        println!("Real lock:{}", l);
    }
    real_locks*/
}