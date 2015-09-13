use std::fmt;
use std::hash;
use std::hash::Hasher;
use super::Queue;

pub use self::Node::*;

// a node in SuffixTree
#[derive(Debug, Hash)]
pub enum Node {
    Leaf { start: usize },
    Internal {
        start: usize,
        end: usize,
        children: Vec<Node>
    },
    Root { children: Vec<Node> }
}

impl Node {
    pub fn root() -> Node {
        Root { children: vec![] }
    }

    pub fn leaf(start: usize) -> Node {
        Leaf { start: start }
    }

    pub fn add_child(&mut self, x: Node) {
        match *self {
            Root { ref mut children } => children.push(x),
            Internal { ref mut children, .. } => children.push(x),
            Leaf { .. } => panic!("leaf can't have a child")
        }
    }

    pub fn slice<'a, T>(&self, seq: &'a [T]) -> &'a [T] {
        match *self {
            Internal { start, end, .. } => &seq[start..end],
            Leaf { start } => &seq[start..],
            _ => panic!("root can't seq")
        }
    }

    pub fn iter_children<'t>(&'t self) -> ::std::slice::Iter<'t, Node> {
        match *self {
            Root { ref children } => children.iter(),
            Internal { ref children, .. } => children.iter(),
            Leaf { .. } => panic!("leaf have no children")
        }
    }

    pub fn is_leaf(&self) -> bool {
        if let Leaf { .. } = *self { true } else { false }
    }

    pub fn is_root(&self) -> bool {
        if let Root { .. } = *self { true } else { false }
    }

    pub fn is_internal(&self) -> bool {
        if let Internal { .. } = *self { true } else { false }
    }
}

pub struct SuffixTree<'a, T: Sized + 'a> {
    orig: &'a [T],
    root: Node
}

impl<'a, T: Eq> SuffixTree<'a, T> {
    pub fn new(string: &'a [T]) -> SuffixTree<'a, T> {
        let mut root = Node::root();
        for (i, c) in string.iter().enumerate() {
            root.add_child(Node::leaf(i))
        }
        SuffixTree {
            orig: string,
            root: root
        }
    }
}

fn hash<T: hash::Hash>(x: &T) -> u64 {
    let mut hasher = hash::SipHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

impl<'a, T: fmt::Display + fmt::Debug> SuffixTree<'a, T> {
    fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");
        dot.push_str("node [shape=point];\n");
        let mut queue = Queue::new();
        queue.enqueue(&self.root);
        let mut pid = 0;
        while !queue.is_empty() {
            let x = queue.dequeue().unwrap();
            let nid = hash(x);
            if x.is_leaf() {
                dot.push_str(&format!("{} -> {} [ label = \"{:?}\"];\n", pid, nid, x.slice(self.orig)));
            } else if x.is_root() || x.is_internal() {
                pid = nid;
                for node in x.iter_children() {
                    let nid = hash(node);
                    dot.push_str(&format!("{} -> {} [ label = \"{:?}\"];\n", pid, nid, node.slice(self.orig)));
                    if node.is_internal() {
                        queue.enqueue(node);
                    }
                }
            }
        }
        dot.push_str("}\n");
        dot
    }
}

impl<'a, T: fmt::Display> fmt::Display for SuffixTree<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SuffixTree {:?}", self.root)
    }
}


#[test]
fn test_suffix_tree() {
    let st = SuffixTree::new(b"apple");
    println!("got => {}", st);
    println!("dot =>\n{}", st.to_dot());
}
