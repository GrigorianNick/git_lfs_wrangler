use core::fmt;

pub struct LfsLock {
    pub file: String,
    pub owner: String,
    pub id: u32,
    pub branch: Option<String>,
    pub dir: Option<String>,
    pub queue: Vec<String>,
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
        }
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