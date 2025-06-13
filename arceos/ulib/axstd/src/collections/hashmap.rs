use alloc::vec;
use alloc::vec::Vec;
use axhal::misc::random;
use core::hash::{Hash, Hasher};
//Hash 代表能被哈希， Hasher 代表哈希算法

pub struct HashMap<K, V> {
    seed: u64, //随机种子，初始化之后就不会变了
    buckets: Vec<Vec<(K, V)>>,
    capacity: usize, //容量
    len: usize //当前键值对个数，超出容量需要扩容
}

impl<'a, K, V> HashMap<K, V>
where K: Hash + Eq + Clone, V: Clone
//Hash 通过 Key 来产生哈希值, Eq 用来比较 Key 是否相等
{
    pub fn new() -> Self {
        Self {
            seed: (random() & 0xFFFF_FFFF_FFFF_FFFF) as u64,
            buckets: vec![Vec::new(); 16],
            capacity: 16,
            len: 0
        }
    }

    fn hash_key(&self, key: &K) -> u64 {
        let mut hasher = SimpleHasher::new(self.seed);
        key.hash(&mut hasher); //使用标准库的 Hash trait
        let hash_value = hasher.finish();
        hash_value % (self.capacity as u64)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.len * 4 >= self.capacity * 3 { // 负载因子超过 0.75，需要扩容
            self.resize();
        }

        let index = self.hash_key(&key);
        for entry in &mut self.buckets[index as usize] {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.buckets[index as usize].push((key, value));
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }
    
    // 测例中需要用到迭代器
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            buckets: &self.buckets,
            bucket_index: 0, //第几个桶
            item_index: 0,   //当前桶里面第几个元素
        }
    }

    /// 给 self.buckets 扩容
    /// 因为capacity 变了，所以需要重新 Hash 一遍
    fn resize(&mut self) {
        let new_capacity = self.capacity * 2;
        let mut new_buckets = Vec::new();
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }
    
        for bucket in &self.buckets {
            for (key, value) in bucket {
                let mut hasher = SimpleHasher::new(self.seed);
                key.hash(&mut hasher);
                let index = (hasher.finish() as usize) % new_capacity;
                new_buckets[index].push((key.clone(), value.clone())); // 需要 Clone
            }
        }
    
        self.buckets = new_buckets; //更新之后，旧的自动被清除
        self.capacity = new_capacity;
    }
}


struct SimpleHasher {
    hash: u64,
}

impl SimpleHasher {
    fn new(seed: u64) -> Self {
        Self { hash: seed }
    }
}

//实现哈希算法
impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= *byte as u64;//步骤 1：当前哈希值与输入字节做异或（增加输入依赖）
            self.hash = self.hash.wrapping_mul(0x100000001b3);// 步骤 2：乘以一个大质数（扩散输入熵）FNV-1a 哈希算法
        }
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}


/// 实现迭代器结构体
pub struct Iter<'a, K, V> {
    buckets: &'a Vec<Vec<(K, V)>>,
    bucket_index: usize,
    item_index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket_index < self.buckets.len() {// 遍历整个 HashMap
            let bucket = &self.buckets[self.bucket_index]; 
            if self.item_index < bucket.len() {//遍历整个桶
                let (ref key, ref value) = bucket[self.item_index];
                self.item_index += 1;
                return Some((key, value));
            } else {
                self.bucket_index += 1;
                self.item_index = 0;
            }
        }
        None //遍历HashMap 结束，返回 None
    }
}

