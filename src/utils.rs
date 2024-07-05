use std::alloc::LayoutError;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{alloc, ops};

use hashbrown::HashMap;
use instant::Instant;
use libc::c_void;
use refpool::{Pool, PoolRef};

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

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:x}", self.id)
    }
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

pub trait TypeDisplay {
    fn type_name() -> &'static str;
}

pub struct UncheckedArray<T> {
    begin: *mut T,
}

impl<T> UncheckedArray<T> {
    pub fn new(size: usize) -> Option<UncheckedArray<T>> {
        unsafe {
            let begin = libc::malloc(size * std::mem::size_of::<T>()) as *mut T;
            if begin.is_null() {
                return None;
            }
            Some(UncheckedArray { begin })
        }
    }

    pub fn get(&self, index: usize) -> &T {
        unsafe { &*self.begin.add(index) }
    }

    pub fn set(&self, index: usize, value: T) {
        unsafe { *self.begin.add(index) = value };
    }
}

impl<T> Drop for UncheckedArray<T> {
    fn drop(&mut self) {
        unsafe { libc::free(self.begin as *mut c_void) };
    }
}

pub struct StableMap<K, V> {
    map: HashMap<K, PoolRef<V>>,
    pool: Pool<V>,
}

impl Default for StableMap<u32, u32> {
    fn default() -> Self {
        Self::new()
    }
}

#[profiling::all_functions]
impl<K: Eq + Hash + Clone, V> StableMap<K, V> {
    pub fn new() -> Self {
        Self { map: HashMap::new(), pool: Pool::new(1024) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { map: HashMap::with_capacity(capacity), pool: Pool::new(capacity) }
    }

    pub fn insert(&mut self, key: &K, value: V) -> PoolRef<V> {
        self.map.insert(key.clone(), PoolRef::new(&self.pool, value));
        self.map.get(key).unwrap().clone()
    }

    /// This function is dangerous.
    /// It is undefined behaviour to use the returned reference after the value is removed or the map is destroyed.
    /// It is also undefined behaviour to change the value while there is a reference to it alive.
    pub fn insert_raw<'b>(&mut self, key: &K, value: V) -> &'b V {
        self.map.insert(key.clone(), PoolRef::new(&self.pool, value));
        self.get_raw(key).unwrap()
    }

    pub fn get(&self, key: &K) -> Option<PoolRef<V>> {
        self.map.get(key).cloned()
    }

    /// This function is dangerous.
    /// It is undefined behaviour to use the returned reference after the value is removed or the map is destroyed.
    /// It is also undefined behaviour to change the value while there is a reference to it alive.
    pub fn get_raw<'b>(&self, key: &K) -> Option<&'b V> {
        let ptr = (self.map.get(key)?.as_ref()) as *const V;
        Some(unsafe { &*ptr })
    }

    pub fn remove(&mut self, key: &K) -> Option<PoolRef<V>> {
        self.map.remove(key)
    }
}

/*pub struct Range<T: Index<usize> + IndexMut<usize> + Ord> {
    range: (T, T),
    len: u32,
}

impl<T: Index<usize> + IndexMut<usize> + Ord> Range<T> {
    pub fn new(range: (T, T), len: u32) -> Range<T> {
        Range { range, len }
    }

    pub fn from<U>(min: U, max: U, len: u32) -> Range<T>
    where
        U: Into<T> + Ord,
    {
        Range { range: (min.into(), max.into()), len }
    }

    pub fn contains<U>(&self, value: U) -> bool
    where
        U: Into<T> + Ord,
    {
        let value = value.into();
        for index in 0..self.len {
            let index = index as usize;
            let (min, max) = self.range;
            if value[index] < min[index] && value[index] > max[index] {
                return false;
            }
        }

        true
    }

    pub fn contains_nth(&self, nth: u32, value: f32) -> bool {
        if nth >= self.len {
            return false;
        }

        let (min, max) = self.data[nth];
        value >= min && value <= max
    }

    pub fn intersects(&self, other: &Range<T>) -> bool {
        if other.len != self.len {
            return false;
        }

        let (min, max) = self.range;
        if !other.contains(min) && !other.contains(max) {
            return false;
        }

        true
    }
}
*/
/*pub struct Entity {}

pub enum Node<'a, T> {
    Empty,
    Inner(Range<f32>, SmallVec<[&'a Node<'a, T>; 8]>),
    Leaf(SmallVec<[T; 8]>),
}*/
