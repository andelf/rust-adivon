use std::mem;
use std::fmt;
use std::ptr;
use std::iter;

use rand::{thread_rng, Rng};

// 32
const DEFAULT_LEVEL: usize = 3;


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
    prev: Rawlink<SkipNode<Key,E>>,
    // level 0 to DEFAULT_LEVEL
    forward: Vec<Rawlink<SkipNode<Key,E>>>,
}


impl<Key: PartialOrd, E> SkipNode<Key,E> {
    // level: 0 ~ DEFAULT_LEVEL
    fn new(key: Key, it: E, level: usize) -> SkipNode<Key,E> {
        SkipNode {
            key: key,
            it: it,
            next: None,
            prev: Rawlink::none(),
            forward: iter::repeat(Rawlink::none()).take(level).collect(),
        }
    }

    #[inline]
    fn level(&self) -> usize {
        self.forward.len()
    }

    fn to_ptr(&mut self) -> Rawlink<SkipNode<Key,E>> {
        Rawlink::some(self)
    }

    fn insert(&mut self, key: Key, it: E, level: usize) {
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
                x = nx
            }

        x.resolve().map_or(None, |n| {
            if n.key == *key {
                Some(&n.it)
            } else {
                None
            }
        })
    }

    fn adjust_head(&mut self, new_level: usize) {
        let diff = new_level - self.level;
        if diff > 0 {
            self.forward.append(&mut iter::repeat(Rawlink::none()).take(diff).collect())
        }
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
                println!("update => {:?}", update);
                y = self.head.as_mut().map_or_else(Rawlink::none, |n| {
                    Rawlink::some(&mut **n)
                });
            }
            while y.resolve().map_or(false, |n| n.next.is_some()) &&
                y.resolve().map_or(false, |n| n.next.as_ref().unwrap().key < key) {
                    let ny = y.resolve_mut().map_or_else(Rawlink::none, |n| {
                        n.next.as_mut().map_or_else(Rawlink::none, |n| Rawlink::some(&mut **n))
                    });
                    y = ny
                }
            assert!(y.is_some());
            // create node and insert
            let mut new_node = Some(Box::new(SkipNode::new(key, it, self.level)));
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

    pub fn remove(&mut self, key: &Key) {
        unimplemented!()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}


impl<Key: PartialOrd + fmt::Display + fmt::Debug, E: fmt::Debug> fmt::Display for SkipList<Key,E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.head.is_none() {
            write!(f, "<empty skip list>")
        } else {
            write!(f, "\nlv0 ");
            let mut x = self.head.as_ref();
            while x.is_some() {
                write!(f, "{} --> ", x.as_ref().map_or_else(String::new, |n| format!("{}", n.key)));
                x = x.as_ref().map_or(None, |n| n.next.as_ref());
            }
            writeln!(f, "");
            for i in 0..self.level {
                write!(f, "lv{} ", i+1);
                let mut x = self.forward[i];
                while x.is_some() {
                    write!(f, "{} ..> ",
                           x.resolve().map_or_else(String::new, |n| format!("{}", n.key)));
                    if x.resolve().map_or(false, |n| n.level() <= i) {
                        break;
                    }
                    x = x.resolve().map_or_else(Rawlink::none, |n| n.forward[i]);
                }
                writeln!(f, "");
            }
            Ok(())
        }
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

    fn take(&mut self) -> Rawlink<T> {
        mem::replace(self, Rawlink::none())
    }

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
    for i in 0 .. 10 {
        let val = rng.gen_range(0, 2000);
        // let val = vals[i];
        println!("DEBUG {} insert => {}", i+1, val);
        list.insert(val, ());
    }

    println!("list => {}", list);
    println!("level => {}", list.level());

    list.insert(20, ());
    println!("has(20) => {}", list.contains_key(&20));
}
