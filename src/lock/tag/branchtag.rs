use crate::lock::LfsLock;
use crate::lock::tag::Tag;

use regex::Regex;

pub struct BranchTag {
    branch: String,
    target_id: u32,
}

impl BranchTag {
    pub fn from_lock(lock: &LfsLock) -> Option<impl Tag> {
        let re = Regex::new("B(?<id>[0-9]+)___(?<branch>.*)").unwrap();
        match re.captures(&lock.file) {
            None => None,
            Some(c) => {
                match (c.name("id"), c.name("branch")) {
                    (Some(id), Some(branch)) => {
                        Some(BranchTag {
                                branch: branch.as_str().to_string(),
                                target_id: id.as_str().parse::<u32>().unwrap(),
                            }
                        )
                    },
                    _ => None,
                }
            },
        }
    }
}

pub fn for_lock(lock: &LfsLock) -> BranchTag {
    BranchTag{
        branch: crate::git::get_branch(),
        target_id: lock.id,
    }
}

impl Tag for BranchTag {

    fn get_lock_string(&self) -> String {
        ["B", self.get_target_id().to_string().as_str(), "___", self.branch.as_str()].join("")
    }

    fn apply(&self, lock: &mut LfsLock) {
        lock.branch = Some(self.branch.clone());
    }

    fn get_target_id(&self) -> u32 {
        self.target_id
    }
}