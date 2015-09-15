use std::fmt;
use std::mem;
use std::ptr;

use std::collections::btree_map::BTreeMap;
use super::Queue;

pub use self::Node::*;

struct Rawlink<T> {
    p: *mut T,
}

impl<T> Copy for Rawlink<T> {}

impl<T> Clone for Rawlink<T> {
    fn clone(&self) -> Rawlink<T> { *self }
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

    fn is_null(&self) -> bool {
        self.p.is_null()
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
// a node in SuffixTree
enum Node<T> {
    Leaf { start: usize },
    Internal {
        start: usize,
        end: usize,
        children: BTreeMap<T, Node<T>>,
        suffix_link: Rawlink<Node<T>>
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

    pub fn internal(start: usize, end: usize) -> Node<T> {
        Internal { start: start, end: end, children: BTreeMap::new(), suffix_link: Rawlink::none() }
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

    fn shrink(&mut self, offset: usize) {
        match *self {
            Leaf { ref mut start } => {
                *start += offset;
            },
            Internal { ref mut start, ref end, .. } => {
                *start += offset;
                assert!(*start > *end);
            },
            Root { .. } => panic!("can't shrink root node")
        }
    }

    fn start(&self) -> usize {
        match *self {
            Leaf { start } => {
                start
            },
            Internal { start, .. } => {
                start
            },
            Root { .. } => 0
        }
    }

    pub fn split_at(&mut self, offset: usize, seq: &[T]) {
        let new = Node::internal(self.start(), self.start()+offset);
        let mut old = mem::replace(self, new);
        old.shrink(offset);
        self.add_child(old, seq);
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

    fn length(&self, pos: usize) -> usize {
        match *self {
            Leaf { ref start } => pos - start,
            Internal { ref start, ref end, .. } => end - start,
            Root { .. } => panic!("leaf have no children")
        }
    }

    fn add_suffix_link(&mut self, slink: Rawlink<Node<T>>) {
        match *self {
            Internal { ref mut suffix_link, .. } => {
                *suffix_link = slink;
            }
            _ => {}
        }
    }

    fn suffix_link(&self) -> Rawlink<Node<T>> {
        match *self {
            Internal { suffix_link, .. } => {
                suffix_link.clone()
            }
            _ => {
                Rawlink::none()
            }
        }
    }

    pub fn mut_child_starts_with<'t>(&'t mut self, c: &T) -> Option<&'t mut Node<T>> {
        match *self {
            Root { ref mut children } => children.get_mut(c),
            Internal { ref mut children, .. } => children.get_mut(c),
            Leaf { .. } => panic!("leaf have no children")
        }
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

impl<'a, T: Ord + Copy> SuffixTree<'a, T> {
    pub fn new(text: &'a [T]) -> SuffixTree<'a, T> {
        let mut st = SuffixTree {
            orig: text,
            root: Node::root()
        };
        st.build();
        st
    }

    // http://stackoverflow.com/questions/9452701/ukkonens-suffix-tree-algorithm-in-plain-english
    // http://pastie.org/5925812
    fn build(&mut self) {
        let root_link = Rawlink::some(&mut self.root);
        let text = self.orig;
        // active point
        let mut active_node = root_link;
        let mut active_edge: usize = 0; //  0 used for null
        let mut active_length = 0;
        // how many to be inserted
        let mut remainder = 0;
        for (pos, &c) in text.iter().enumerate() {
            remainder += 1;
            // println!("active point => ({:?}, {:?}, {})", active_node.resolve(), active_edge, active_length);
            // println!("remainder => {}", remainder);
            let mut need_suffix_link: Rawlink<Node<T>> = Rawlink::none();

            while remainder > 0 {
                if active_length == 0 { active_edge = pos }
                if active_node.resolve().map_or(false, |n| n.child_starts_with(&text[active_edge]).is_none()) {
                    active_node.resolve_mut().map(|n| n.add_child(Node::leaf(pos), text));
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                    need_suffix_link = active_node;
                } else if let Some(ref mut next) = active_node.resolve_mut().unwrap().mut_child_starts_with(&text[active_edge]) {
                    // walk down
                    if active_length >= next.length(pos) {
                        active_edge += next.length(pos);
                        active_length -= next.length(pos);
                        active_node = Rawlink::some(next);
                        continue;
                    }
                    if text[next.start() + active_length] == c {
                        active_length += 1;
                        need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                        break;
                    }
                    next.split_at(active_length, text);
                    next.add_child(Node::leaf(pos), text);
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(Rawlink::some(next)));
                    need_suffix_link = Rawlink::some(next);
                }
                remainder -= 1;

                if active_node.resolve().unwrap().is_root() && active_length > 0 { // rule 1
                    active_length -= 1;
                    active_edge = pos - remainder + 1;
                } else {
                    // rule 3
                    let link = active_node.resolve().unwrap().suffix_link();
                    if link.is_null() {
                        active_node = root_link;
                    } else {
                        active_node = link;
                    }

                }
            }
        }
    }
}


fn dot_id<T>(x: &T) -> u64 {
    unsafe {
        mem::transmute::<_, u64>(x)
    }
}

impl<'a, T: Ord + Copy + fmt::Display> SuffixTree<'a, T> {
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");
        dot.push_str("  node [shape=point];\n");
        let mut queue = Queue::new();
        queue.enqueue(&self.root);
        while !queue.is_empty() {
            let x = queue.dequeue().unwrap();
            let pid = dot_id(x);
            if x.is_leaf() {

            } else if x.is_root() || x.is_internal() {
                for node in x.iter_children() {
                    let nid = dot_id(node);
                    dot.push_str(&format!("  {} -> {} [ label = \"{}\"];\n", pid, nid, node.slice(self.orig).iter().map(|c| c.to_string()).collect::<Vec<String>>().join(" ")));
                    x.suffix_link().resolve().map(|n| dot.push_str(&format!("  {} -> {} [ style=dashed ];\n", pid, dot_id(n))));
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
        write!(f, "SuffixTree(orig: {:?})", self.orig)
    }
}


#[test]
fn test_suffix_tree() {
    let s = "abcabxabcd".chars().collect::<Vec<char>>();
    let st = SuffixTree::new(&s);
    println!("==================================================");
    println!("got => {}", st);
    println!("dot =>\n{}", st.to_dot());
}
