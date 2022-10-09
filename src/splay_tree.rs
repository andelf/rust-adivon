use std::cmp;
use std::fmt;
use std::iter;

fn compare<T: PartialOrd>(a: &T, b: &T) -> i32 {
    match a.partial_cmp(b).unwrap() {
        cmp::Ordering::Greater => 1,
        cmp::Ordering::Less => -1,
        _ => 0,
    }
}

pub type NodeCell<K, V> = Option<Box<Node<K, V>>>;

pub struct Node<K, V> {
    left: NodeCell<K, V>,
    right: NodeCell<K, V>,
    key: K,
    val: V,
}

impl<K: fmt::Debug, V: fmt::Debug> Node<K, V> {
    fn dump(&self, depth: usize, f: &mut fmt::Formatter, symbol: char) {
        if depth == 0 {
            writeln!(f, "\n{:?}[{:?}]", self.key, self.val).unwrap();
        } else {
            writeln!(
                f,
                "{}{}--{:?}[{:?}]",
                iter::repeat("|  ").take(depth - 1).collect::<Vec<&str>>().concat(),
                symbol,
                self.key,
                self.val
            )
            .unwrap();
        }
        if self.left.is_some() {
            self.left.as_ref().unwrap().dump(depth + 1, f, '+');
        }
        if self.right.is_some() {
            self.right.as_ref().unwrap().dump(depth + 1, f, '`');
        }
    }
}

impl<K: PartialOrd, V> Node<K, V> {
    fn new(key: K, val: V) -> Node<K, V> {
        Node {
            left: None,
            right: None,
            key,
            val,
        }
    }

    fn height(x: Option<&Node<K, V>>) -> usize {
        if let Some(node) = x {
            let lh = Node::height(node.left.as_deref());
            let rh = Node::height(node.left.as_deref());
            if lh <= rh {
                rh + 1
            } else {
                lh + 1
            }
        } else {
            0
        }
    }

    fn size(x: Option<&Node<K, V>>) -> usize {
        if let Some(node) = x {
            1 + Node::size(node.left.as_deref()) + Node::size(node.right.as_deref())
        } else {
            0
        }
    }

    fn splay(mut h: NodeCell<K, V>, key: &K) -> NodeCell<K, V> {
        h.as_ref()?;
        let cmp1 = h.as_ref().map(|n| compare(key, &n.key)).unwrap();
        match cmp1.cmp(&0) {
            cmp::Ordering::Less => {
                // key not in tree, done
                if h.as_ref().unwrap().left.is_none() {
                    return h;
                }
                let cmp2 = compare(key, &h.as_ref().unwrap().left.as_ref().unwrap().key);
                match cmp2.cmp(&0) {
                    cmp::Ordering::Less => {
                        h.as_mut().map(|n| {
                            n.left.as_mut().map(|n| {
                                n.left = Node::splay(n.left.take(), key);
                            })
                        });
                        h = Node::rotate_right(h);
                    }
                    cmp::Ordering::Greater => {
                        if let Some(ref mut n) = h.as_mut() {
                            let f = |n: &mut Box<Node<K, V>>| {
                                n.right = Node::splay(n.right.take(), key);
                                n.right.is_some()
                            };
                            if n.left.as_mut().map_or(false, f) {
                                n.left = Node::rotate_left(n.left.take());
                            }
                        }
                    }
                    _ => {}
                }
                // key not in tree
                if h.as_ref().unwrap().left.is_none() {
                    return h;
                } else {
                    return Node::rotate_right(h);
                }
            }
            cmp::Ordering::Greater => {
                // key not in tree, done
                if h.as_ref().unwrap().right.is_none() {
                    return h;
                }
                let cmp2 = compare(key, &h.as_ref().unwrap().right.as_ref().unwrap().key);
                match cmp2.cmp(&0) {
                    cmp::Ordering::Less => {
                        if let Some(n) = h.as_mut() {
                            let f = |n: &mut Box<Node<K, V>>| {
                                n.left = Node::splay(n.left.take(), key);
                                n.left.is_some()
                            };
                            if n.right.as_mut().map_or(false, f) {
                                n.right = Node::rotate_right(n.right.take());
                            }
                        }
                    }
                    cmp::Ordering::Greater => {
                        h.as_mut().map(|n| {
                            n.right.as_mut().map(|n| {
                                n.right = Node::splay(n.right.take(), key);
                            })
                        });
                        h = Node::rotate_left(h);
                    }
                    _ => {}
                }
                // key not in tree
                if h.as_ref().unwrap().right.is_none() {
                    return h;
                } else {
                    return Node::rotate_left(h);
                }
            }
            _ => {}
        }
        h
    }

    fn rotate_right(mut h: NodeCell<K, V>) -> NodeCell<K, V> {
        let mut x = h.as_mut().and_then(|n| n.left.take());
        if let Some(n) = h.as_mut() {
            n.left = x.as_mut().and_then(|n| n.right.take());
        }
        if let Some(n) = x.as_mut() {
            n.right = h;
        }
        x
    }

    fn rotate_left(mut h: NodeCell<K, V>) -> NodeCell<K, V> {
        let mut x = h.as_mut().and_then(|n| n.right.take());
        if let Some(n) = h.as_mut() {
            n.right = x.as_mut().and_then(|n| n.left.take());
        }
        if let Some(n) = x.as_mut() {
            n.left = h;
        }
        x
    }
}

/// Splay tree. Supports splay-insert, -search, and -delete.
pub struct SplayTree<K, V> {
    root: NodeCell<K, V>,
    // size: usize
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for SplayTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.root.is_none() {
            write!(f, "<empty tree>")
        } else {
            self.root.as_ref().unwrap().dump(0, f, ' ');
            Ok(())
        }
    }
}

impl<K: PartialOrd, V> Default for SplayTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialOrd, V> SplayTree<K, V> {
    pub fn new() -> SplayTree<K, V> {
        SplayTree {
            root: None,
            // size: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn size(&self) -> usize {
        Node::size(self.root.as_deref())
    }

    pub fn height(&self) -> usize {
        Node::height(self.root.as_deref())
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    // get() needs to update tree structure
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.root = Node::splay(self.root.take(), key);
        self.root
            .as_ref()
            .and_then(|n| if n.key == *key { Some(&n.val) } else { None })
    }

    pub fn contains_key(&mut self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn get_mut<'t>(&'t mut self, key: &K) -> Option<&'t mut V> {
        self.root = Node::splay(self.root.take(), key);
        self.root
            .as_mut()
            .and_then(|n| if n.key == *key { Some(&mut n.val) } else { None })
    }

    /// Splay tree insertion.
    pub fn insert(&mut self, key: K, val: V) {
        if self.root.is_none() {
            self.root = Some(Box::new(Node::new(key, val)));
            return;
        }

        let mut root = Node::splay(self.root.take(), &key);

        let cmp = compare(&key, &root.as_ref().unwrap().key);
        match cmp.cmp(&0) {
            std::cmp::Ordering::Less => {
                let mut n = Node::new(key, val);
                n.left = root.as_mut().unwrap().left.take();
                n.right = root;
                self.root = Some(Box::new(n));
            }
            std::cmp::Ordering::Greater => {
                let mut n = Node::new(key, val);
                n.right = root.as_mut().unwrap().right.take();
                n.left = root;
                self.root = Some(Box::new(n));
            }
            std::cmp::Ordering::Equal => {
                if let Some(n) = root.as_mut() {
                    n.val = val;
                }
                self.root = root;
            }
        }
    }

    /// Splay tree deletion.
    // use Algs4 approach
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.root.as_ref()?;

        let mut root = Node::splay(self.root.take(), key);

        if *key == root.as_ref().unwrap().key {
            if root.as_ref().unwrap().left.is_none() {
                self.root = root.as_mut().unwrap().right.take();
            } else {
                let x = root.as_mut().unwrap().right.take();
                self.root = Node::splay(root.as_mut().unwrap().left.take(), key);
                if let Some(n) = self.root.as_mut() {
                    n.right = x;
                }
            }
            root.map(|n| n.val)
        } else {
            None
        }
    }
}

#[test]
fn test_splay_tree() {
    let mut st1 = SplayTree::new();
    assert!(st1.is_empty());
    st1.insert(5, 5);
    st1.insert(9, 9);
    st1.insert(13, 13);
    st1.insert(11, 11);
    st1.insert(1, 1);

    assert_eq!(5, st1.size());
    assert!(st1.height() < 5);
    for i in [5, 9, 13, 11, 1].iter() {
        assert_eq!(st1.get(i), Some(i));
    }

    st1.get_mut(&1).map(|val| *val = 1000);
    assert_eq!(st1.get(&1), Some(&1000));

    assert!(st1.remove(&9).is_some());
    assert!(st1.remove(&9).is_none());
    assert!(!st1.contains_key(&9));
}
