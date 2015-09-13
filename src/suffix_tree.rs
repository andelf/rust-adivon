use std::fmt;
use std::hash;
use std::hash::Hasher;

use std::collections::btree_map::BTreeMap;
use super::Queue;

pub use self::Node::*;

// a node in SuffixTree
#[derive(Debug, Hash)]
pub enum Node<T> {
    Leaf { start: usize },
    Internal {
        start: usize,
        end: usize,
        children: BTreeMap<T, Node<T>>
    },
    Root { children: BTreeMap<T, Node<T>> }
}

impl<T: Ord + Copy> Node<T> {
    pub fn root() -> Node<T> {
        Root { children: BTreeMap::new() }
    }

    pub fn leaf(start: usize) -> Node<T> {
        Leaf { start: start }
    }

    pub fn add_child(&mut self, x: Node<T>, string: &[T]) {
        match *self {
            Root { ref mut children } => {
                children.insert(x.head(string), x);
            },
            Internal { ref mut children, .. } => {
                children.insert(x.head(string), x);
            },
            Leaf { .. } => panic!("leaf can't have a child")
        }
    }

    pub fn slice<'a>(&self, seq: &'a [T]) -> &'a [T] {
        match *self {
            Internal { start, end, .. } => &seq[start..end],
            Leaf { start } => &seq[start..],
            _ => panic!("root can't seq")
        }
    }

    pub fn head(&self, seq: &[T]) -> T {
        match *self {
            Internal { start, .. } => seq[start],
            Leaf { start } => seq[start],
            _ => panic!("root have no head")
        }
    }

    pub fn iter_children<'t>(&'t self) -> ::std::collections::btree_map::Values<'t, T, Node<T>> {
        match *self {
            Root { ref children } => children.values(),
            Internal { ref children, .. } => children.values(),
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

    pub fn child_starts_with(&self, c: &T) -> Option<&Node<T>> {
        match *self {
            Root { ref children } => children.get(c),
            Internal { ref children, .. } => children.get(c),
            Leaf { .. } => panic!("leaf have no children")
        }
    }
}

pub struct SuffixTree<'a, T: Sized + 'a> {
    orig: &'a [T],
    root: Node<T>
}

impl<'a, T: Ord + Copy + fmt::Debug> SuffixTree<'a, T> {
    pub fn new(string: &'a [T]) -> SuffixTree<'a, T> {
        let mut root = Node::root();
        {
            // active point
            let mut active_node = &mut root;
            let mut active_edge: Option<T> = None;
            let mut active_length = 0;
            // how many to be inserted
            let mut remainder = 0;

            for (i, &c) in string.iter().enumerate() {
                println!("i=> {} c => {:?}", i, c);
                println!("node => {:?}", active_node);
                if active_edge.map_or(false, |e| active_node.child_starts_with(&e).map_or(false, |n| n.slice(string)[active_length] == c )) {
                    println!("in e");
                } else if active_node.child_starts_with(&c).is_some() {
                    active_edge = Some(c);
                    active_length += 1;
                    remainder += 1;
                }

                if active_edge.is_none() {
                    active_node.add_child(Node::leaf(i), string);
                }
            }

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

impl<'a> SuffixTree<'a, u8> {
    pub fn to_dot(&self) -> String {
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

            } else if x.is_root() || x.is_internal() {
                pid = nid;
                for node in x.iter_children() {
                    let nid = hash(node);
                    dot.push_str(&format!("{} -> {} [ label = \"{:}\"];\n", pid, nid, String::from_utf8_lossy(node.slice(self.orig))));
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

impl<'a, T: fmt::Display + fmt::Debug> fmt::Display for SuffixTree<'a, T> {
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
