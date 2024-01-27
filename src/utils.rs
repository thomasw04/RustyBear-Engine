use std::collections::HashSet;
use std::hash::Hash;
use std::ops;
use std::path::{Path, PathBuf};

use instant::Instant;

pub struct Timestep {
    delta: f64,
    last: Instant,
    begin: Instant,
}

impl Default for Timestep {
    fn default() -> Self {
        let begin = Instant::now();

        Timestep { delta: 0.0, last: begin, begin }
    }
}

impl Timestep {
    pub fn step_fwd(&mut self) -> &mut Self {
        self.delta = self.last.elapsed().as_nanos() as f64 / 1000000.0;
        self.last = Instant::now();
        self
    }

    pub fn norm(&self) -> f32 {
        (self.delta / 10.0) as f32
    }

    pub fn micros(&self) -> i64 {
        (self.delta * 1000.0) as i64
    }

    pub fn millis(&self) -> f64 {
        self.delta
    }

    pub fn seconds(&self) -> f64 {
        self.delta / 1000.0
    }

    pub fn total_secs(&self) -> f64 {
        self.begin.elapsed().as_secs_f64()
    }
}

impl From<f64> for Timestep {
    fn from(delta: f64) -> Timestep {
        let begin = Instant::now();

        Timestep { delta, last: begin, begin }
    }
}

impl From<Timestep> for f64 {
    fn from(value: Timestep) -> f64 {
        value.delta
    }
}

impl ops::AddAssign<f64> for Timestep {
    fn add_assign(&mut self, rhs: f64) {
        self.delta += rhs;
    }
}

impl ops::SubAssign<f64> for Timestep {
    fn sub_assign(&mut self, rhs: f64) {
        self.delta -= rhs;
    }
}

impl ops::MulAssign<f64> for Timestep {
    fn mul_assign(&mut self, rhs: f64) {
        self.delta *= rhs;
    }
}

impl ops::DivAssign<f64> for Timestep {
    fn div_assign(&mut self, rhs: f64) {
        self.delta /= rhs;
    }
}

pub struct FileUtils {}

impl FileUtils {
    pub fn find_ext_in_dir(root_dir: &Path, ext: &str) -> Option<PathBuf> {
        if !root_dir.is_dir() {
            return None;
        }

        let files_result = std::fs::read_dir(root_dir);

        match files_result {
            Err(error) => {
                log::error!(
                    "Could not look into directory {}. Message: {}",
                    root_dir.to_str().unwrap_or("ERR_NON_UTF8_PATH"),
                    error
                );

                None
            }
            Ok(mut files) => {
                let file_result = files.find(|file_result| match file_result {
                    Ok(file) => FileUtils::has_extension(file.path().as_path(), ext),
                    Err(error) => {
                        log::error!(
                            "A file error occurred while in {}. Message: {}",
                            root_dir.to_str().unwrap_or("ERR_NON_UTF8_PATH"),
                            error
                        );
                        false
                    }
                });

                file_result.and_then(|res| res.ok().map(|file| file.path()))
            }
        }
    }

    pub fn has_extension(file: &Path, ext: &str) -> bool {
        if !file.is_file() {
            return false;
        }

        file.extension().and_then(|s| s.to_str()).is_some_and(|extension| extension.eq(ext))
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Guid {
    id: u64,
}

impl Guid {
    pub fn new(id: u64) -> Guid {
        Guid { id }
    }

    pub fn is_dead(&self) -> bool {
        self.id == 0
    }

    pub fn dead() -> Guid {
        Guid { id: 0 }
    }
}

#[derive(Default)]
pub struct GuidGenerator {
    used: HashSet<u64>,
}

impl GuidGenerator {
    pub fn new() -> GuidGenerator {
        Default::default()
    }

    pub fn generate(&mut self) -> Guid {
        let mut id = rand::random::<u64>();
        const RESERVED_IDS: u64 = 10;
        while self.used.contains(&id) && id < RESERVED_IDS {
            id = rand::random::<u64>();
        }
        self.used.insert(id);
        Guid::new(id)
    }
}
