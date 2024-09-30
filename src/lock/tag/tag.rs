use regex::Regex;

use crate::lock::*;
use crate::lock::tag::*;

pub trait Tag {
    // Update a lock's info
    fn apply(&self, lock: &mut LfsLock);
    // Save the relevant info to the lfs
    fn save(&self);
    // Get the id of the lock this tag is associated with
    fn get_target_id(&self) -> u32;
    // Apply and save
    fn tag(&self, lock: &mut LfsLock) {
        self.apply(lock);
        self.save();
    }
}

type TagCtor = fn(&LfsLock) -> Option<Box<dyn Tag>>;

struct TagFactory {
    ctors: &'static [TagCtor],
}

static factory: &'static TagFactory = &TagFactory {
    ctors: &[dirtag::DirTag::from_lock,
    branchtag::BranchTag::from_lock,
    queuetag::QueueTag::from_lock],
};

//impl TagFactory {
    // If a lock is a tag, then we hand back a tag. If it doesn't, None
    pub fn get_tag(lock: &LfsLock) -> Option<Box<dyn Tag>> {
        for f in factory.ctors {
            match f(lock) {
                None => (),
                Some(tag) => {
                    return Some(tag);
                }
            }
        }
        None
    }
//}