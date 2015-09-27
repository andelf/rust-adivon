use std::fmt;
use std::ptr;
use std::iter;

use std::collections::BTreeMap;

use rand::{thread_rng, Rng};


const DEFAULT_LEVEL: usize = 4;

type Link<T> = Option<Box<T>>;

#[allow(raw_pointer_derive)]
#[derive(Debug)]
struct Rawlink<T> {
    p: *mut T,
}

impl<T> Copy for Rawlink<T> {}
unsafe impl<T:Send> Send for Rawlink<T> {}
unsafe impl<T:Sync> Sync for Rawlink<T> {}


fn random_level() -> usize {
    let mut rng = thread_rng();
    let mut level = 0;
    loop {
        if rng.gen_weighted_bool(2) {
            level += 1;
            continue;
        }
        break;
    }

    level
}

#[derive(Debug)]
pub struct SkipNode<Key,E> {
    key: Key,
    it: E,
    next: Link<SkipNode<Key,E>>,
    // level 0 to DEFAULT_LEVEL
    forward: Vec<Rawlink<SkipNode<Key,E>>>,
    level: usize
}


impl<Key: PartialOrd, E> SkipNode<Key,E> {
    // level: 0 ~ DEFAULT_LEVEL
    fn new(key: Key, it: E, level: usize) -> SkipNode<Key,E> {
        SkipNode {
            key: key,
            it: it,
            next: None,
            forward: iter::repeat(Rawlink::none()).take(level).collect(),
            level: level
        }
    }

    #[inline]
    fn level(&self) -> usize {
        self.forward.len()
    }

    fn promote_level(&mut self, new_level: usize, forward: Vec<Rawlink<Self>>) {
        let level = self.level();
        for i in level .. new_level {
            self.forward.push(forward[i]);
        }
        assert!(self.level() == new_level, "promote_level()");
    }
}

pub struct SkipList<Key,E> {
    head: Link<SkipNode<Key,E>>,
    forward: Vec<Rawlink<SkipNode<Key,E>>>,
    level: usize,
    size: usize,
}


impl<Key: PartialOrd + fmt::Debug, E: fmt::Debug> SkipList<Key,E> {
    /// create new empty SkipList
    pub fn new() -> SkipList<Key, E> {
        SkipList {
            head: None,
            forward: iter::repeat(Rawlink::none()).take(DEFAULT_LEVEL).collect(),
            level: DEFAULT_LEVEL,
            size: 0,
        }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.find(key).is_some()
    }

    pub fn find(&self, key: &Key) -> Option<&E> {
        let level = self.level();
        let mut x = self.forward[level-1];
        for i in (0..level).rev() {
            while x.resolve().map_or(false, |n| n.forward[i].is_some()) &&
                x.resolve().map_or(false, |n| n.forward[i].resolve().unwrap().key < *key) {
                    let nx = x.resolve().map_or_else(Rawlink::none, |n| n.forward[i] );
                    x = nx;
                }
        }

        while x.resolve().map_or(false, |n| n.next.is_some()) &&
            x.resolve().map_or(false, |n| n.next.as_ref().unwrap().key < *key) {
                let nx = x.resolve_mut().map_or_else(Rawlink::none, |n| {
                    n.next.as_mut().map_or_else(Rawlink::none, |n| Rawlink::some(&mut **n))
                });
                x = nx;
            }

        // current x.key is lower than key
        // jump next
        x = x.resolve_mut().map_or_else(Rawlink::none, |n| Rawlink::from(&mut n.next));
        x.resolve().map_or(None, |n| {
            if n.key == *key {
                Some(&n.it)
            } else {
                None
            }
        })
    }

    fn adjust_head(&mut self, new_level: usize) {
        let head_link  = Rawlink::from(&mut self.head);
        let level = self.level();

        for _ in level .. new_level {
            self.forward.push(head_link);
            self.head.as_mut().map(|n| n.forward.push(Rawlink::none()));
        }
        assert_eq!(self.forward.len(), new_level);
        assert_eq!(self.head.as_ref().map_or(new_level, |n| n.level()), new_level);
    }

    // Due to head node must be of same level as List,
    // inserting with decreasing order will lead to almost same bad performance as a linked list
    pub fn insert(&mut self, key: Key, it: E) {
        let mut new_level = random_level();
        if new_level > self.level { // new node will be deepest
            self.adjust_head(new_level);
            self.level = new_level;
        }
        self.size += 1;

        if self.head.is_none() {
            new_level = self.level;
            self.head = Some(Box::new(SkipNode::new(key, it, new_level)));
            let p = Rawlink::from(&mut self.head);
            for i in 0..new_level {
                self.forward[i] = p;
            }
        } else if self.head.as_ref().map_or(false, |n| n.key > key) {
            new_level = self.level;
            // insert at head
            let mut new_node = Some(Box::new(SkipNode::new(key, it, new_level)));
            let new_link = new_node.as_mut().map_or_else(Rawlink::none, |n| {
                Rawlink::some(&mut **n)
            });

            for i in 0..new_level {
                new_node.as_mut().map(|n| {
                    n.forward[i] = self.forward[i]
                });
            }

            for i in 0..new_level {
                self.forward[i] = new_link
            }

            new_node.as_mut().map(|n| n.next = self.head.take());
            self.head = new_node;
        } else {
            let mut x = self.head.as_mut().map_or_else(Rawlink::none, |n| {
                Rawlink::some(&mut **n)
            });
            // insert normally
            let mut update: Vec<Rawlink<SkipNode<Key,E>>> = self.forward[..new_level].to_vec();
            if new_level > 0 {  // need to insert skip pointers
                x = update[new_level-1];
                for i in (0..new_level).rev() {
                    while x.resolve().map_or(false, |n| n.forward[i].is_some()) &&
                        x.resolve().map_or(false, |n| n.forward[i].resolve().unwrap().key < key) {
                            let nx = x.resolve().map_or_else(Rawlink::none, |n| n.forward[i] );
                            x = nx;
                        }
                    update[i] = x.resolve_mut().map_or_else(Rawlink::none, |n| {
                        Rawlink::some(&mut *n)
                    });
                }
            }

            let mut y = x.resolve_mut().map_or_else(Rawlink::none, |n| {
                Rawlink::some(&mut *n)
            });
            // When head node level is lower than current
            if y.is_none() {
                y = self.head.as_mut().map_or_else(Rawlink::none, |n| {
                    Rawlink::some(&mut **n)
                });
            }
            while y.resolve().map_or(false, |n| n.next.is_some()) &&
                y.resolve().map_or(false, |n| n.next.as_ref().unwrap().key < key) {
                    y = y.resolve_mut().map_or_else(Rawlink::none, |n| Rawlink::from(&mut n.next));
                }
            assert!(y.is_some());
            // create node and insert
            let mut new_node = Some(Box::new(SkipNode::new(key, it, new_level)));
            let new_link = new_node.as_mut().map_or_else(Rawlink::none, |n| {
                Rawlink::some(&mut **n)
            });

            for i in 0..new_level {
                new_node.as_mut().map(|n| {
                    n.forward[i] = update[i].resolve_mut().map_or_else(Rawlink::none, |nx| {
                        nx.forward[i]
                    });
                });
                // if update is empty, then update head node's link
                update[i].resolve_mut().map_or_else(|| {
                    self.forward[i] = new_link;
                },
                    |prev| {
                    prev.forward[i] = new_link;
                });
            }

            // move in
            y.resolve_mut().map(|n| {
                new_node.as_mut().map(|new| {
                    new.next = n.next.take();
                });
                n.next = new_node;
            });
        }
    }

    pub fn remove(&mut self, key: &Key) -> Option<E> {
        if self.head.is_none() {
            return None;
        }
        let level = self.level();
        let mut x = self.forward[level-1];

        let mut update: Vec<Rawlink<SkipNode<Key,E>>> = self.forward[..level].to_vec();

        for i in (0..level).rev() {
            while x.resolve().map_or(false, |n| n.forward[i].is_some()) &&
                x.resolve().map_or(false, |n| n.forward[i].resolve().unwrap().key < *key) {
                    x = x.resolve().map_or_else(Rawlink::none, |n| n.forward[i]);
                }
            update[i] = x.resolve_mut().map_or_else(Rawlink::none, |n| Rawlink::some(&mut *n));
        }

        while x.resolve().map_or(false, |n| n.next.is_some()) &&
            x.resolve().map_or(false, |n| n.next.as_ref().unwrap().key < *key) {
                x = x.resolve_mut().map_or_else(Rawlink::none, |n| Rawlink::from(&mut n.next));
            }


        if x.resolve().map_or(false, |n| n.key == *key ) {
            // key is head item
            let head = self.head.take().unwrap();
            let head = *head;   // unwrap Box
            let SkipNode { it, next, forward, .. } = head;
            let mut new_head = next;

            // calculate new level, means, only head nodes
            let new_level = forward.iter().take_while(|nx| nx.is_some()).count();

            self.forward = iter::repeat(Rawlink::from(&mut new_head)).take(new_level).collect();
            self.level = new_level;

            new_head.as_mut().map(|n| n.promote_level(new_level, forward));
            self.head = new_head;

            Some(it)
        } else if x.is_some() {
            // to be deleted
            let current = x.resolve_mut().map_or(None, |n| n.next.take());
            if current.as_ref().map_or(true, |n| n.key != *key ) {
                return None;
            }
            let level = current.as_ref().unwrap().level();
            // destruct
            let SkipNode { it, next, forward, .. } = *current.unwrap();
            // move next in
            x.resolve_mut().map(|n| n.next = next);
            // chain prev node and next node
            for (i, prev) in update.iter_mut().take(level).enumerate() {
                prev.resolve_mut().map(|n| {
                    n.forward[i] = forward[i];
                });
            }
            Some(it)
        } else {                // x is empty
            None
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }
}


impl<Key: Ord + fmt::Display + fmt::Debug, E: fmt::Debug> fmt::Display for SkipList<Key,E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.head.is_none() {
            return write!(f, "<empty skip list>")
        }
        try!(write!(f, "\nlv0  "));

        let mut offset_map = BTreeMap::new();
        let mut offset = 5;
        let mut x = self.head.as_ref();
        while x.is_some() {
            // FIXME: change to unwrap() at last format
            offset_map.insert(&x.as_ref().unwrap().key, offset);
            let nx = x.as_ref().map_or(None, |n| n.next.as_ref());
            let label = x.as_ref().map_or_else(String::new, |n| format!("{}", n.key));
            if nx.is_none() {
                try!(writeln!(f, "{}", label));
            } else {
                try!(write!(f, "{} --> ", label));
            }
            x = nx;
            offset += label.len() + 5;
        }
        for i in 0..self.level {
            try!(write!(f, "lv{:<2} ", i+1));
            offset = 5;
            let mut x = self.forward[i];
            while x.is_some() {
                let label = x.resolve().map_or_else(String::new, |n| format!("{}", n.key));
                let lv0_pos = offset_map.get(&x.resolve().unwrap().key).unwrap();
                let padding = lv0_pos  - offset;

                if offset == 5 { // fist item
                    try!(write!(f, "{} ", label));
                } else {
                    try!(write!(f, "{}> {} ", iter::repeat('.').take(padding).collect::<String>(), label));
                }
                x = x.resolve().map_or_else(Rawlink::none, |n| n.forward[i]);
                offset = lv0_pos + label.len() + 3;
            }
            try!(writeln!(f, ""));
        }
        Ok(())
    }
}

/// Rawlink is a type like Option<T> but for holding a raw mutable pointer.
impl<T> Rawlink<T> {
    /// Like `Option::None` for Rawlink.
    fn none() -> Rawlink<T> {
        Rawlink{p: ptr::null_mut()}
    }

    /// Like `Option::Some` for Rawlink
    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink{p: n}
    }

    fn is_some(&self) -> bool {
        !self.is_none()
    }

    fn is_none(&self) -> bool {
        self.p.is_null()
    }

    // fn take(&mut self) -> Rawlink<T> {
    //     mem::replace(self, Rawlink::none())
    // }

    /// Convert the `Rawlink` into an immutable Option value.
    fn resolve<'a>(&self) -> Option<&'a T> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe { &*self.p })
        }
    }

    /// Convert the `Rawlink` into a mutable Option value.
    fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe { &mut *self.p })
        }
    }
}

impl<'a, Key, E> From<&'a mut Link<SkipNode<Key, E>>> for Rawlink<SkipNode<Key, E>> {
    fn from(node: &'a mut Link<SkipNode<Key, E>>) -> Self {
        match node.as_mut() {
            None => Rawlink::none(),
            Some(ptr) => Rawlink::some(&mut **ptr),
        }
    }
}


impl<T> Clone for Rawlink<T> {
    #[inline]
    fn clone(&self) -> Rawlink<T> {
        Rawlink{p: self.p}
    }
}



#[test]
fn test_skip_list() {
    let mut list: SkipList<i32, ()> = SkipList::new();

    let mut rng = thread_rng();

    //let vals = vec![ -18130, 16865, -1813, 1686, -181, 168, -18, 16];
    for i in 0 .. 20 {
        let val = rng.gen_range(0, 2000);
        // let val = vals[i];
        println!("DEBUG {} insert => {}", i+1, val);
        list.insert(val, ());
    }

    println!("list => {}", list);
    println!("level => {}", list.level());

    let v = 1000;
    list.insert(v, ());
    println!("list => {}", list);
    assert!(list.contains_key(&1000));
    assert!(!list.contains_key(&3000));

    list.remove(&v);
    assert!(!list.contains_key(&1000));
    println!("list => {}", list);
}
