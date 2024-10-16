use std::env;

use crate::lock::LfsLock;
use regex::Regex;
use crate::lock::tag::Tag;

pub struct DirTag {
    target_id: u32,
    dir: String,
}


impl DirTag {
    // build a tag from its lfs lock representation
    pub fn from_lock(lock: &LfsLock) -> Option<DirTag> {
        let dir_re: Regex = Regex::new("D(?<id>[0-9]+)___(?<dir>.*)").unwrap();
        match dir_re.captures(&lock.file) {
            None => None,
            Some(capture) => {
                match (capture.name("id"), capture.name("dir")) {
                    (Some(id), Some(dir)) => {
                        Some(DirTag{
                            target_id: id.as_str().parse::<u32>().expect("Failed to parse int"),
                            dir: dir.as_str().to_string(),
                        })
                    }
                    _ => None,
                }
            }
        }
    }
}

pub fn for_lock(lock: &LfsLock) -> Box<DirTag> {
    Box::new(
        DirTag {
            target_id: lock.id,
            dir: env::current_dir().expect("cwd is mangled").to_string_lossy().to_string(),
        }
    )
}

impl Tag for DirTag {
    fn apply(&self, lock: &mut LfsLock) {
        lock.dir = Some(self.dir.clone());
    }

    fn get_lock_string(&self) -> String {
        ["D", self.get_target_id().to_string().as_str(), "___", self.dir.as_str()].join("")
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }
}