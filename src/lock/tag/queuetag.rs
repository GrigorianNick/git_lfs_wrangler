use crate::lock::{lock, LfsLock};
use super::Tag;

use regex::Regex;

pub struct QueueTag {
    target_id: u32,
    target_file: String,
}

pub fn for_lock(lock: &LfsLock) -> Box<QueueTag> {
    Box::new(
        QueueTag {
            target_id: lock.id,
            target_file: lock.file.clone(),
        }
    )
}

impl QueueTag {
    pub fn from_lock(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        let re = Regex::new(r"Q(?<id>[0-9]+)___(?<file>.*)").expect("Regex failed to compile");
        match re.captures(&lock.file) {
            None => None,
            Some(c) =>  {
                match (c.name("id"), c.name("file")) {
                    (Some(id), Some(f)) => Some(Box::new(QueueTag{
                        target_id: id.as_str().parse().expect("failed to parse int"),
                        target_file: f.as_str().to_string(),
                    })),
                    _ => None,
                }
            }
        }
    }
}

impl Tag for QueueTag {
    fn save(&self) {
        let lock_path = ["Q", self.target_id.to_string().as_str(), "___", self.target_file.as_str()].join("");
        lock::LfsLock::lock_file(&lock_path);
    }

    fn apply(&self, lock: &mut LfsLock) {
        lock.queue.push(self.target_id);
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }
}