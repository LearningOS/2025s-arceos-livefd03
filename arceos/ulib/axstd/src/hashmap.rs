extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::mem;

// 假设的hash函数，你可以替换为你自己的实现
fn hash<T: Hash>(key: &T) -> usize {
    use core::hash::SipHasher;
    let mut hasher = SipHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as usize
}

// HashMap的条目
struct Entry<K, V> {
    key: K,
    value: V,
    next: Option<Box<Entry<K, V>>>,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, value: V) -> Self {
        Entry {
            key,
            value,
            next: None,
        }
    }
}

// HashMap结构
pub struct HashMap<K, V> {
    buckets: Vec<Option<Box<Entry<K, V>>>>,
    size: usize,
    capacity: usize,
}

// 不可变迭代器
pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket_idx: usize,
    current: Option<&'a Entry<K, V>>,
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone
{
    // 创建一个新的HashMap
    pub fn new() -> Self {
        let initial_capacity = 16;
        let mut buckets = Vec::with_capacity(initial_capacity);
        buckets.resize_with(initial_capacity, || None);

        HashMap {
            buckets,
            size: 0,
            capacity: initial_capacity,
        }
    }

    // 插入键值对
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.size >= self.capacity * 3 / 4 {
            self.resize();
        }
    
        let index = hash(&key) % self.capacity;
        let mut entry = self.buckets[index].take();
    
        // 检查是否已存在相同的 key
        let mut prev = None;
        let mut current = entry;
        let mut old_value = None;
    
        while let Some(mut boxed_entry) = current {
            if boxed_entry.key == key {
                old_value = Some(mem::replace(&mut boxed_entry.value, value.clone()));
                current = boxed_entry.next.take();
                break;
            }
            prev = Some(boxed_entry);
            current = prev.as_mut().unwrap().next.take();
        }
    
        // 重建链表
        if let Some(mut p) = prev {
            p.next = current;
            entry = Some(p);
        } else {
            entry = current;
        }
    
        // 如果 key 不存在，添加新条目
        if old_value.is_none() {
            let mut new_entry = Box::new(Entry::new(key, value));
            new_entry.next = entry;
            self.buckets[index] = Some(new_entry);
            self.size += 1;
        } else {
            self.buckets[index] = entry;
        }
    
        old_value
    }

    // 获取值
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = hash(key) % self.capacity;
        let mut current = self.buckets[index].as_ref();

        while let Some(entry) = current {
            if &entry.key == key {
                return Some(&entry.value);
            }
            current = entry.next.as_ref();
        }

        None
    }

    // 获取可变值
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = hash(key) % self.capacity;
        let mut current = self.buckets[index].as_mut();

        while let Some(entry) = current {
            if &entry.key == key {
                return Some(&mut entry.value);
            }
            current = entry.next.as_mut();
        }

        None
    }

    // 移除键值对
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = hash(key) % self.capacity;
        let mut entry = self.buckets[index].take();
        let mut prev = None;
        let mut current = entry;
        let mut removed = None;

        while let Some(mut boxed_entry) = current {
            if &boxed_entry.key == key {
                removed = Some(boxed_entry.value);
                current = boxed_entry.next.take();
                self.size -= 1;
                break;
            }
            prev = Some(boxed_entry);
            current = prev.as_mut().unwrap().next.take();
        }

        // 重建链表
        if let Some(mut p) = prev {
            p.next = current;
            entry = Some(p);
        } else {
            entry = current;
        }

        self.buckets[index] = entry;
        removed
    }

    // 扩容哈希表
    fn resize(&mut self) {
        let new_capacity = self.capacity * 2;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        new_buckets.resize_with(new_capacity, || None);

        for bucket in self.buckets.drain(..) {
            let mut current = bucket;
            while let Some(mut entry) = current {
                let next = entry.next.take();
                let index = hash(&entry.key) % new_capacity;
                
                entry.next = new_buckets[index].take();
                new_buckets[index] = Some(entry);
                
                current = next;
            }
        }

        self.buckets = new_buckets;
        self.capacity = new_capacity;
    }

    // 获取大小
    pub fn len(&self) -> usize {
        self.size
    }

    // 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    // 返回不可变迭代器
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            map: self,
            bucket_idx: 0,
            current: None,
        }
    }
}

// 实现不可变迭代器
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // 如果当前条目存在，返回它并移动到下一个
            if let Some(entry) = self.current {
                let result = (&entry.key, &entry.value);
                self.current = entry.next.as_ref().map(|b| &**b);
                return Some(result);
            }

            // 否则查找下一个非空桶
            if self.bucket_idx >= self.map.buckets.len() {
                return None;
            }

            self.current = self.map.buckets[self.bucket_idx].as_ref().map(|b| &**b);
            self.bucket_idx += 1;
        }
    }
}
