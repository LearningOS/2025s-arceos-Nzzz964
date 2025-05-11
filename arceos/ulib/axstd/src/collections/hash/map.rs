use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub trait Hash {
    fn hash<H: Hasher>(&self, state: &mut H);

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.hash(state)
        }
    }
}

impl Hash for &str {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
    }
}

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
    }
}

pub trait Hasher {
    fn finish(&self) -> u64;

    fn write(&mut self, bytes: &[u8]);

    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_ne_bytes())
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_ne_bytes())
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_ne_bytes())
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_ne_bytes())
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_ne_bytes())
    }

    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8)
    }

    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16)
    }

    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32)
    }

    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64)
    }

    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128)
    }

    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize)
    }

    fn write_length_prefix(&mut self, len: usize) {
        self.write_usize(len);
    }

    fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
        self.write_u8(0xff);
    }
}

impl<H: Hasher + ?Sized> Hasher for &mut H {
    fn finish(&self) -> u64 {
        (**self).finish()
    }
    fn write(&mut self, bytes: &[u8]) {
        (**self).write(bytes)
    }
    fn write_u8(&mut self, i: u8) {
        (**self).write_u8(i)
    }
    fn write_u16(&mut self, i: u16) {
        (**self).write_u16(i)
    }
    fn write_u32(&mut self, i: u32) {
        (**self).write_u32(i)
    }
    fn write_u64(&mut self, i: u64) {
        (**self).write_u64(i)
    }
    fn write_u128(&mut self, i: u128) {
        (**self).write_u128(i)
    }
    fn write_usize(&mut self, i: usize) {
        (**self).write_usize(i)
    }
    fn write_i8(&mut self, i: i8) {
        (**self).write_i8(i)
    }
    fn write_i16(&mut self, i: i16) {
        (**self).write_i16(i)
    }
    fn write_i32(&mut self, i: i32) {
        (**self).write_i32(i)
    }
    fn write_i64(&mut self, i: i64) {
        (**self).write_i64(i)
    }
    fn write_i128(&mut self, i: i128) {
        (**self).write_i128(i)
    }
    fn write_isize(&mut self, i: isize) {
        (**self).write_isize(i)
    }
    fn write_length_prefix(&mut self, len: usize) {
        (**self).write_length_prefix(len)
    }
    fn write_str(&mut self, s: &str) {
        (**self).write_str(s)
    }
}

pub struct FNV1aHasher(u64);

impl FNV1aHasher {
    pub fn new() -> Self {
        Self(0xcbf29ce484222325)
    }
}

impl Default for FNV1aHasher {
    fn default() -> Self {
        FNV1aHasher::new() // 调用已有的 new 方法
    }
}

impl Hasher for FNV1aHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

static DEFAULT_CAPACITY: usize = 50_000 + 10;

pub struct HashMap<K, V> {
    // buckets
    buckets: Vec<Option<(K, V)>>,
    // 哈希表容量
    capacity: usize,
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Eq + Clone,
{
    pub fn new() -> HashMap<K, V> {
        Self::new_with_capacity(None)
    }

    pub fn new_with_capacity(capacity: Option<usize>) -> HashMap<K, V> {
        let cap = capacity.map_or(DEFAULT_CAPACITY, |x| x);
        let bucket = vec![None; cap];
        Self {
            buckets: bucket,
            capacity: cap,
        }
    }

    fn hash(&self, k: &K) -> u64 {
        let mut hasher = FNV1aHasher::default();
        k.hash(&mut hasher);
        let hash = hasher.finish();
        hash % self.capacity as u64
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let mut index = self.hash(&k) as usize;

        for _ in 0..self.capacity {
            match self.buckets[index] {
                // key 已经存在，更新值
                Some((ref sk, ref mut sv)) if *sk == k => {
                    return Some(core::mem::replace(sv, v));
                }
                // index 冲突，线性探测法
                Some(_) => {
                    index = (index + 1) % self.capacity;
                }
                // 没有冲突
                ref mut slot @ None => {
                    *slot = Some((k, v));
                    return None;
                }
            }
        }

        panic!("HashMap is full. Resizing not implemented.");
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            index: 0,
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let mut index = self.hash(k) as usize;
        for _ in 0..self.capacity {
            match &self.buckets[index] {
                Some((ref sk, ref sv)) if sk == k => {
                    return Some(sv);
                }
                Some(_) => {
                    index = (index + 1) % self.capacity;
                }
                None => {
                    return None;
                }
            }
        }
        None
    }
}

pub struct Iter<'a, K, V> {
    buckets: &'a Vec<Option<(K, V)>>,
    index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Eq + Hash + Clone,
    V: Eq + Clone,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.buckets.len() {
            if let Some((ref k, ref v)) = self.buckets[self.index] {
                self.index += 1;
                return Some((k, v));
            }
            self.index += 1;
        }
        None
    }
}
