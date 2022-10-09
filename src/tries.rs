use super::Queue;

pub struct Node<V, K: Copy + PartialOrd = char> {
    c: K,
    left: Option<Box<Node<V, K>>>,
    mid: Option<Box<Node<V, K>>>,
    right: Option<Box<Node<V, K>>>,
    val: Option<V>,
}

impl<K: PartialOrd + Copy, V> Node<V, K> {
    fn new(c: K) -> Node<V, K> {
        Node {
            c,
            left: None,
            mid: None,
            right: None,
            val: None,
        }
    }

    fn put(
        mut x: Option<Box<Node<V, K>>>,
        key: &[K],
        val: Option<V>,
        d: usize,
    ) -> (Option<Box<Node<V, K>>>, Option<V>) {
        let replaced;
        let c = key[d];
        if x.is_none() {
            if val.is_none() {
                // no need to call put further
                return (x, None);
            }
            x = Some(Box::new(Node::new(c)));
        }
        let xc = x.as_ref().unwrap().c;
        if c < xc {
            let (left, repl) = Node::put(x.as_mut().unwrap().left.take(), key, val, d);
            if let Some(n) = x.as_mut() {
                n.left = left;
            }
            replaced = repl;
        } else if c > xc {
            let (right, repl) = Node::put(x.as_mut().unwrap().right.take(), key, val, d);
            if let Some(n) = x.as_mut() {
                n.right = right;
            }
            replaced = repl;
        } else if d < key.len() - 1 {
            let (mid, repl) = Node::put(x.as_mut().unwrap().mid.take(), key, val, d + 1);
            if let Some(n) = x.as_mut() {
                n.mid = mid;
            }
            replaced = repl;
        } else {
            replaced = x.as_mut().unwrap().val.take();
            if let Some(n) = x.as_mut() {
                n.val = val;
            }
        }
        (x, replaced)
    }

    fn get<'a>(x: Option<&'a Node<V, K>>, key: &[K], d: usize) -> Option<&'a Node<V, K>> {
        x?;
        let c = key[d];
        let xc = x.unwrap().c;
        if c < xc {
            Node::get(x.unwrap().left.as_deref(), key, d)
        } else if c > xc {
            Node::get(x.unwrap().right.as_deref(), key, d)
        } else if d < key.len() - 1 {
            Node::get(x.unwrap().mid.as_deref(), key, d + 1)
        } else {
            x
        }
    }

    fn get_mut<'a>(x: Option<&'a mut Box<Node<V, K>>>, key: &[K], d: usize) -> Option<&'a mut Box<Node<V, K>>> {
        x.as_ref()?;
        let c = key[d];
        let xc = x.as_ref().unwrap().c;
        if c < xc {
            Node::get_mut(x.unwrap().left.as_mut(), key, d)
        } else if c > xc {
            Node::get_mut(x.unwrap().right.as_mut(), key, d)
        } else if d < key.len() - 1 {
            Node::get_mut(x.unwrap().mid.as_mut(), key, d + 1)
        } else {
            x
        }
    }

    fn collect(x: Option<&Node<V, K>>, mut prefix: Vec<K>, queue: &mut Queue<Vec<K>>) {
        if x.is_none() {
            return;
        }
        Node::collect(x.unwrap().left.as_deref(), prefix.clone(), queue);
        let xc = x.unwrap().c;
        prefix.push(xc);
        if x.unwrap().val.is_some() {
            queue.enqueue(prefix.clone());
        }
        Node::collect(x.unwrap().mid.as_deref(), prefix.clone(), queue);
        prefix.pop();
        Node::collect(x.unwrap().right.as_deref(), prefix, queue);
    }

    fn longest_prefix_of<'a>(mut x: Option<&Node<V, K>>, query: &'a [K]) -> Option<&'a [K]> {
        let mut length = 0;
        let mut i = 0;
        while x.is_some() && i < query.len() {
            let c = query[i];
            let xc = x.unwrap().c;
            if c < xc {
                x = x.unwrap().left.as_deref();
            } else if c > xc {
                x = x.unwrap().right.as_deref();
            } else {
                i += 1;
                if x.unwrap().val.is_some() {
                    length = i;
                }
                x = x.unwrap().mid.as_deref();
            }
        }
        if length == 0 {
            None
        } else {
            Some(&query[..length])
        }
    }
}

/// Symbol table with string keys, implemented using a ternary search trie (TST).
pub struct TernarySearchTrie<V, K: PartialOrd + Copy = char> {
    root: Option<Box<Node<V, K>>>,
    n: usize,
}

impl<K: PartialOrd + Copy, V> Default for TernarySearchTrie<V, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialOrd + Copy, V> TernarySearchTrie<V, K> {
    pub fn new() -> TernarySearchTrie<V, K> {
        TernarySearchTrie { root: None, n: 0 }
    }

    pub fn put(&mut self, key: &[K], val: V) {
        let (root, replaced) = Node::put(self.root.take(), key, Some(val), 0);
        self.root = root;
        // replace old val? or insert new?
        if replaced.is_none() {
            self.n += 1;
        }
    }

    pub fn get(&self, key: &[K]) -> Option<&V> {
        assert!(!key.is_empty(), "key must have length >= 1");
        Node::get(self.root.as_deref(), key, 0).and_then(|n| n.val.as_ref())
    }

    pub fn get_mut(&mut self, key: &[K]) -> Option<&mut V> {
        assert!(!key.is_empty(), "key must have length >= 1");
        Node::get_mut(self.root.as_mut(), key, 0).and_then(|n| n.val.as_mut())
    }

    pub fn delete(&mut self, key: &[K]) {
        let (root, replaced) = Node::put(self.root.take(), key, None, 0);
        self.root = root;
        // deleted?
        if replaced.is_some() {
            self.n -= 1;
        }
    }

    pub fn size(&self) -> usize {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn contains(&self, key: &[K]) -> bool {
        self.get(key).is_some()
    }

    pub fn longest_prefix_of<'a>(&self, query: &'a [K]) -> Option<&'a [K]> {
        Node::longest_prefix_of(self.root.as_deref(), query)
    }

    pub fn keys_with_prefix(&self, prefix: &[K]) -> Vec<Vec<K>> {
        let mut queue = Queue::new();
        let x = Node::get(self.root.as_deref(), prefix, 0);
        if let Some(x) = x {
            if x.val.is_some() {
                queue.enqueue(prefix.into());
            }
            Node::collect(x.mid.as_deref(), prefix.into(), &mut queue);
        }
        queue.into_iter().collect()
    }

    pub fn keys(&self) -> Vec<Vec<K>> {
        let mut queue = Queue::new();
        Node::collect(self.root.as_deref(), vec![], &mut queue);
        queue.into_iter().collect()
    }
}

#[test]
fn test_tst() {
    let mut t = TernarySearchTrie::new();
    assert_eq!(t.size(), 0);
    t.put(b"name", "Andelf");
    assert_eq!(t.size(), 1);
    assert!(t.contains(b"name"));
    t.put(b"name", "Fledna");
    assert_eq!(t.size(), 1);
    t.put(b"language", "Rust");
    assert_eq!(t.size(), 2);

    assert_eq!(t.get(b"name"), Some(&"Fledna"));
    assert_eq!(t.get(b"whatever"), None);

    t.delete(b"name");
    assert_eq!(t.size(), 1);
    assert_eq!(t.get(b"name"), None);

    t.put(b"name", "Lednaf");
    assert!(t.keys().contains(&"name".into()));
    assert!(t.keys().contains(&"language".into()));
    assert!(t.keys_with_prefix(b"lang").contains(&"language".into()));

    t.put(b"ban", "2333");
    t.put(b"banana", "2333");
    assert_eq!(t.longest_prefix_of(b"bananananana").unwrap(), b"banana");

    t.get_mut(b"banana").map(|v| *v = "46666");
    assert_eq!(t.get(b"banana").unwrap(), &"46666");
}
