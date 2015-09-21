use std::fmt;
use std::mem;
use std::iter;
use std::ptr;
use std::iter::FromIterator;

use std::collections::{BTreeMap};

use vec_map::VecMap;

use super::{Queue, Stack};

pub use self::Node::*;


fn min<T: PartialOrd + Copy>(x: T, y: T) -> T {
    if x >= y {
        y
    } else {
        x
    }
}



#[allow(raw_pointer_derive)]
#[derive(Debug)]
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

/// a node in SuffixTree
#[derive(Debug, Clone)]
enum Node<'a, T: 'a> {
    Internal {
        /// the edge label
        data: &'a [T],
        /// { text index in root: start position of offset}
        offsets: VecMap<usize>,
        /// text terminates at this node, suffix offset: { text index in root: suffix offset}
        // TODO: use bit vec
        terminates: VecMap<usize>,
        children: BTreeMap<T, Node<'a, T>>,
        suffix_link: Rawlink<Node<'a, T>>
    },
    Root { children: BTreeMap<T, Node<'a, T>> }
}

impl<'a, T: Ord + Copy + fmt::Debug> Node<'a, T> {
    pub fn root() -> Node<'a, T> {
        Root { children: BTreeMap::new() }
    }

    pub fn leaf(data: &'a [T], txt_idx: usize, start_pos: usize, rank: usize) -> Node<'a, T> {
        Internal {
            data: data,
            offsets: VecMap::from_iter(vec![(txt_idx, start_pos)]),
            terminates: VecMap::from_iter(vec![(txt_idx, rank)]),
            children: BTreeMap::new(),
            suffix_link: Rawlink::none()
        }
    }

    // pub fn internal(data: &'a [T], txt_idx: usize, start_pos: usize) -> Node<T> {
    //     Internal {
    //         data: data,
    //         offsets: VecMap::from_iter(vec![(txt_idx, start_pos)]),
    //         terminates: VecMap::new(),
    //         children: BTreeMap::new(),
    //         suffix_link: Rawlink::none()
    //     }
    // }

    pub fn add_child(&mut self, x: Node<'a, T>) {
        match *self {
            Root { ref mut children } => {
                children.insert(x.head(), x);
            },
            Internal { ref mut children, .. } => {
                children.insert(x.head(), x);
            },
        }
    }

    fn truncated_internal(&mut self, txt_idx: usize, offset: usize) -> Node<'a, T> {
        match *self {
            Internal { ref mut data, ref mut offsets, ref mut suffix_link, .. } => {
                let new_data = &data[..offset];
                *data = &data[offset..];
                let mut new_offsets = offsets.clone();
                // update offset info
                new_offsets.insert(txt_idx, offset);
                for (_key, value) in offsets.iter_mut() {
                    *value += offset;
                }

                Internal {
                    data: new_data,
                    offsets: new_offsets,
                    terminates: VecMap::new(),
                    children: BTreeMap::new(),
                    suffix_link: suffix_link.take()
                }
            },
            Root { .. } => panic!("can't truncate a root node")
        }
    }

    pub fn split_at(&mut self, txt_idx: usize, offset: usize) {
        assert!(offset < self.data().len());
        // let new = Node::internal(&self.data()[0..offset], txt_idx);
        let new = self.truncated_internal(txt_idx, offset);
        let old = mem::replace(self, new);
        self.add_child(old);
    }

    pub fn add_terminate(&mut self, txt_idx: usize, position: usize) {
        match *self {
            Internal { ref mut terminates, .. } => {
                terminates.insert(txt_idx, position);
            },
            _ => panic!("add terminate error ")
        }
    }

    pub fn terminates_any(&self) -> bool {
        match *self {
            Internal { ref terminates, .. } => {
                terminates.len() != 0
            }
            _ => panic!("calling terminates_any() on wrong node")
        }
    }

    pub fn terminates(&self, txt_idx: usize) -> Option<usize> {
        match *self {
            Internal { ref terminates, .. } => {
                terminates.get(&txt_idx).map(|&pos| pos)
            },
            _ => panic!("terminates error ")
        }
    }

    #[inline]
    pub fn data(&self) -> &'a [T] {
        match *self {
            Internal { data, .. } => data,
            _ => panic!("root hava no data label")
        }
    }

    pub fn head(&self) -> T {
        match *self {
            Internal { data, .. } => data[0],
            _ => panic!("root have no head")
        }
    }

    pub fn iter_children<'t>(&'t self) -> ::std::collections::btree_map::Values<'t, T, Node<'a, T>> {
        match *self {
            Root { ref children } => children.values(),
            Internal { ref children, .. } => children.values()
        }
    }

    pub fn is_root(&self) -> bool {
        if let Root { .. } = *self { true } else { false }
    }

    pub fn is_internal(&self) -> bool {
        if let Internal { .. } = *self { true } else { false }
    }

    fn length(&self, txt_idx: usize, pos: usize) -> usize {
        match *self {
            Internal { ref data, ref offsets, .. } => {
                let offset = *offsets.get(&txt_idx).unwrap();
                min(pos - offset + 1, data.len())
            },
            Root { .. } => 0,
        }
    }

    fn add_suffix_link(&mut self, slink: Rawlink<Node<'a, T>>) {
        match *self {
            Internal { ref mut suffix_link, .. } => {
                *suffix_link = slink;
            }
            _ => {}
        }
    }

    fn suffix_link(&self) -> Rawlink<Node<'a, T>> {
        match *self {
            Internal { suffix_link, .. } => {
                suffix_link.clone()
            }
            _ => {
                Rawlink::none()
            }
        }
    }

    pub fn mut_child_starts_with<'t>(&'t mut self, c: &T) -> Option<&'t mut Node<'a, T>> {
        match *self {
            Root { ref mut children } => children.get_mut(c),
            Internal { ref mut children, .. } => children.get_mut(c)
        }
    }
    pub fn child_starts_with(&self, c: &T) -> Option<&Node<'a, T>> {
        match *self {
            Root { ref children } => children.get(c),
            Internal { ref children, .. } => children.get(c)
        }
    }
}

#[derive(Debug)]
pub struct SuffixTree<'a, T: Sized + 'a> {
    txts: Vec<&'a [T]>,
    root: Node<'a, T>
}

impl<'a, T: Ord + Copy + fmt::Debug> SuffixTree<'a, T> {
    pub fn new(txt: &'a [T]) -> SuffixTree<'a, T> {
        let mut st = SuffixTree {
            txts: vec![],
            root: Node::root()
        };
        st.add(txt);
        st
    }

    /// check if a string query is a substring
    // pub fn contains(&self, query: &[T]) -> bool {
    //     let text = self.txts;
    //     let mut x = Some(&self.root);
    //     let nquery = query.len();
    //     let mut pos = 0;
    //     while !x.map_or(true, |n| n.is_leaf()) && pos < nquery {
    //         x = x.unwrap().child_starts_with(&query[pos]);
    //         if let Some(ref node) = x {
    //             let label = node.slice();
    //             let nlabel = label.len();
    //             if nlabel <= query[pos..].len() {
    //                 if label == &query[pos.. pos + nlabel] {
    //                     pos += nlabel;
    //                 } else {
    //                     return false;
    //                 }
    //             } else {
    //                 return label.starts_with(&query[pos..]);
    //             }
    //         }
    //     }
    //     pos == nquery
    // }

    pub fn add(&mut self, txt: &'a [T]) {
        self.ukkonen95(txt)
    }

    // http://stackoverflow.com/questions/9452701/ukkonens-suffix-tree-algorithm-in-plain-english
    // http://pastie.org/5925812
    // Ukkonen (1995)
    fn ukkonen95(&mut self, txt: &'a [T]) {
        let root_link = Rawlink::some(&mut self.root);
        let txt_idx = self.txts.len();
        self.txts.push(txt);
        let tlen = txt.len();
        // active point
        let mut active_node = root_link;
        let mut active_edge: usize = 0; //  0 used for null
        let mut active_length = 0;
        // how many to be inserted
        let mut remainder = 0;
        for (pos, &c) in txt.iter().enumerate() {
            remainder += 1;
            let mut need_suffix_link: Rawlink<Node<T>> = Rawlink::none();

            println!("DEBUG running on pos=>{} {:?}", pos, c);
            while remainder > 0 {
                if active_length == 0 { active_edge = pos }
                if active_node.resolve().map_or(false, |n| n.child_starts_with(&txt[active_edge]).is_none()) {
                    active_node.resolve_mut().map(|n| n.add_child(Node::leaf(&txt[pos..], txt_idx, pos, pos)));
                    need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                    need_suffix_link = active_node;
                } else if let Some(ref mut next) = active_node.resolve_mut().unwrap().mut_child_starts_with(&txt[active_edge]) {
                    // walk down
                    if let Internal { ref mut offsets, .. } = **next {
                        if offsets.get(&txt_idx).is_none() {
                            offsets.insert(txt_idx, active_edge);
                        }
                    }

                    let nlen = next.length(txt_idx, pos);

                    if active_length >= nlen {
                        active_edge += nlen;
                        active_length -= nlen;
                        active_node = Rawlink::some(next);
                        continue;
                    }
                    if next.data()[active_length] == c {
                        if pos == tlen - 1 {
                            println!("will split node: {:?}", next);
                            println!("split at {}", active_length + 1);
                            // ends at current node, just add to terminate VecMap
                            if next.data().len() == active_length + 1 {
                                next.split_at(txt_idx, active_length + 1);
                                // need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(Rawlink::some(next)));
                                // need_suffix_link = Rawlink::some(next);
                            }
                            next.add_terminate(txt_idx, pos - active_edge);
                            // active_length += 1;
                            // remainder -= 1;
                            // if remainder > 0 {
                            //     active_length -= 1;
                            // }
                            // continue;
                        } else {
                            active_length += 1;
                            need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(active_node));
                            break;
                        }
                    } else {
                        println!("split at {}", active_length);
                        next.split_at(txt_idx, active_length);
                        next.add_child(Node::leaf(&txt[pos..], txt_idx, pos, pos));
                        need_suffix_link.resolve_mut().map(|n| n.add_suffix_link(Rawlink::some(next)));
                        need_suffix_link = Rawlink::some(next);
                    }
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
            if remainder > 0 {
                println!(":( remainder = {}", remainder);
            }
        }
    }
}


fn dot_id<T>(x: &T) -> u64 {
    unsafe {
        mem::transmute::<_, u64>(x)
    }
}

impl<'a, T: Ord + Copy + fmt::Display + fmt::Debug> SuffixTree<'a, T> {
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph G {\n");
        dot.push_str("  node [shape=point];\n");
        let mut queue = Queue::new();
        queue.enqueue(&self.root);
        while !queue.is_empty() {
            let x = queue.dequeue().unwrap();
            let pid = dot_id(x);
            for node in x.iter_children() {
                let nid = dot_id(node);
                if node.terminates_any() {
                    dot.push_str(&format!("  {} [ color = \"red\", ];\n", nid));
                }
                // FIXME: concat with no space between chars
                dot.push_str(&format!("  {} -> {} [ label = \"{}\"];\n", pid, nid,
                                      node.data().iter().map(|c| c.to_string()).collect::<Vec<String>>().concat()));
                // x.suffix_link().resolve().map(|n| dot.push_str(&format!("  {} -> {} [ style=dashed ];\n", pid, dot_id(n))));
                if node.is_internal() {
                    queue.enqueue(node);
                }
            }

        }
        dot.push_str("}\n");
        dot
    }
}

impl<'a, T: Ord + Copy + fmt::Display + fmt::Debug> fmt::Display for SuffixTree<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SuffixTree(txts: {:?})\n", self.txts);
        let mut stack = Stack::new();
        let mut ident_stack = Stack::new();
        stack.push(&self.root);
        ident_stack.push(0);
        while !stack.is_empty() {
            let x = stack.pop().unwrap();
            let ident = ident_stack.pop().unwrap();
            if !x.is_root() {
                let spaces = String::from_iter(iter::repeat(' ').take(ident).collect::<Vec<char>>());
                write!(f, "{}|<{}>", spaces, x.data().iter().map(|c| c.to_string()).collect::<Vec<String>>().concat());
                if x.terminates_any() {
                    writeln!(f, "*");
                } else {
                    writeln!(f, "");
                }
            }
            for node in x.iter_children(){
                stack.push(node);
                ident_stack.push(ident + 2);
            }
        }
        Ok(())
    }
}


#[test]
fn test_suffix_tree() {
    use std::fs::File;
    use std::io::Write;

    let s1 = "abcabcaba".chars().collect::<Vec<char>>();
    let s2 = "ca".chars().collect::<Vec<char>>();
    println!("s1 => {:?}", s1);
    let mut st = SuffixTree::new(&s1);
    println!("got => {}", st);
    println!("s2 => {:?}", s2);
    st.add(&s2);
    println!("==================================================");
    println!("got => {}", st);
    println!("dot =>\n{}", st.to_dot());
    let mut f = File::create("./stree.dot").unwrap();
    write!(f, "{}", st.to_dot());
}


// #[test]
// fn test_suffix_tree_contains() {
//     let s = b"abcabxabcdaabab";
//     let st = SuffixTree::new(s);

//     assert!(st.contains(b"abc"));
//     assert!(st.contains(b""));
//     assert!(st.contains(b"b"));
//     assert!(!st.contains(b"y"));
//     assert!(st.contains(b"abcabxabcdaabab"));
//     assert!( st.contains(b"bxabcdaa"));
//     assert!(!st.contains(b"bxabadaa"));
// }
