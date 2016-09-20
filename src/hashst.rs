use std::hash::{Hash, Hasher, SipHasher};
use std::borrow::Borrow;
use std::ops::Index;

struct Node<K, V> {
    key: K,
    val: V,
    next: Option<Box<Node<K, V>>>
}

impl<K, V> Node<K, V> {
    pub fn size(&self) -> usize {
        match self.next {
            None => 1,
            Some(ref next) => 1 + next.size()
        }
    }
}

fn delete<K: PartialEq, V>(x: Option<Box<Node<K, V>>> , key: &K) -> Option<Box<Node<K, V>>> {
    if let Some(mut x) = x {
        let next = x.next.take();
        if x.key == *key {
            next            // this will drop x
        } else {
            x.next = delete(next, key);
            Some(x)
        }
    } else {
        None
    }
}

const M: usize = 97;

// separate chaining
pub struct HashST<K, V> {
    st: Vec<Option<Box<Node<K, V>>>>
}


impl<K: Hash + PartialEq, V> HashST<K, V> {
    pub fn new() -> HashST<K, V> {
        let mut st = Vec::with_capacity(M);
        (0..M).map(|_| st.push(None)).count();
        HashST {
            st: st
        }
    }

    // FIXME: hash state bug
    fn hash(key: &K) -> usize {
        let mut hasher = SipHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize % M
    }

    pub fn get<T: Borrow<K>>(&self, key: T) -> Option<&V> {
        let key = key.borrow();
        let i = Self::hash(key);
        let mut x = self.st[i].as_ref();
        while x.is_some() {
            if *key == x.unwrap().key {
                return Some(&x.unwrap().val)
            }
            x = x.unwrap().next.as_ref();
        }
        None
    }

    pub fn get_mut<T: Borrow<K>>(&mut self, key: T) -> Option<&mut V> {
        let key = key.borrow();
        let i = Self::hash(key);
        let mut x = self.st[i].as_mut();
        while x.is_some() {
            if x.as_ref().map_or(false, |n| n.key == *key) {
                return Some(&mut x.unwrap().val)
            }
            x = x.unwrap().next.as_mut();
        }
        None
    }

    pub fn put(&mut self, key: K, val: V) {
        let i = Self::hash(&key);
        {
            let mut x = self.st[i].as_mut();
            while x.is_some() {
                if x.as_ref().map_or(false, |x| x.key == key) {
                    x.unwrap().val = val;
                    return;
                } else {
                    x = x.unwrap().next.as_mut();
                }
            }
        }
        let old = self.st[i].take();
        self.st[i] = Some(Box::new(Node { key: key, val: val, next: old }))
    }

    pub fn delete(&mut self, key: &K) {
        let i = Self::hash(key);
        self.st[i] = delete(self.st[i].take(), key);
    }

    pub fn size(&self) -> usize {
        self.st.iter().filter(|n| n.is_some()).map(|h| h.as_ref().unwrap().size()).sum()
    }
}

// TODO: how to implement IndexMut?
impl<K: Hash + PartialEq, V> Index<K> for HashST<K, V> {
    type Output = V;
    fn index(&self, index: K) -> &V {
        self.get(index).expect("key not exists")
    }
}

#[test]
fn test_separate_chaining_hash_st() {
    let mut m = HashST::new();
    assert_eq!(m.size(), 0);
    m.put("Name", "Feather");
    m.put("Age", "25");
    m.put("Address", "Beijing");

    assert_eq!(m.size(), 3);
    assert_eq!(m.get("Age"), Some(&"25"));
    assert_eq!(m.get("Gender"), None);

    m.delete(&"Age");
    assert_eq!(m.size(), 2);
    assert_eq!(m.get("Age"), None);

    m.get_mut("Address").map(|v| *v = "Shanghai");
    assert_eq!(m.get("Address"), Some(&"Shanghai"));

    assert_eq!(m["Address"], "Shanghai");
}
