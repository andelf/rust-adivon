use std::fmt;
use std::ops;

use self::Rope::*;

fn max<T: PartialOrd + Copy>(x: T, y: T) -> T {
    if x >= y {
        x
    } else {
        y
    }
}

#[derive(Clone)]
pub enum Rope {
    FlatCharVec {
        seq: Vec<char>,
    },
    Concatenation {
        left: Box<Rope>,
        right: Box<Rope>,
        depth: usize,
        length: usize,
    },
    Reverse {
        rope: Box<Rope>,
    },
    SubString {
        rope: Box<Rope>,
        offset: usize,
        length: usize,
    },
}

fn write_rope_to_string(rope: &Rope, s: &mut String) {
    match *rope {
        FlatCharVec { ref seq } => {
            s.push_str(&seq.iter().cloned().collect::<String>());
        }
        Concatenation {
            ref left, ref right, ..
        } => {
            write_rope_to_string(left, s);
            write_rope_to_string(right, s);
        }
        Reverse { ref rope } => {
            let inner = rope.to_string();
            s.push_str(&inner.chars().rev().collect::<String>());
        }
        SubString {
            ref rope,
            offset,
            length,
        } => {
            let inner = rope.to_string();
            s.push_str(&inner[offset..offset + length]);
        }
    }
}

fn concatenate(left: Rope, right: Rope) -> Rope {
    const COMBINE_LENGTH: usize = 17;

    if left.is_empty() {
        return right;
    }
    if right.is_empty() {
        return left;
    }

    let length = left.len() + right.len();
    if length < COMBINE_LENGTH {
        let mut lchs = left.into_chars();
        lchs.extend(right.into_chars());
        return FlatCharVec { seq: lchs };
    }
    let depth = max(left.depth(), right.depth()) + 1;
    Concatenation {
        left: Box::new(left),
        right: Box::new(right),
        depth: depth,
        length: length,
    }
}

impl Rope {
    pub fn from_vec(seq: Vec<char>) -> Rope {
        Rope::FlatCharVec { seq: seq }
    }

    pub fn len(&self) -> usize {
        match *self {
            FlatCharVec { ref seq } => seq.len(),
            Reverse { ref rope } => rope.len(),
            SubString { length, .. } | Concatenation { length, .. } => length,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn depth(&self) -> usize {
        match *self {
            FlatCharVec { .. } => 0,
            Concatenation { depth, .. } => depth,
            Reverse { ref rope } | SubString { ref rope, .. } => rope.depth(),
        }
    }

    pub fn append<RHS: IntoRope>(self, rhs: RHS) -> Self {
        concatenate(self, rhs.into_rope())
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        write_rope_to_string(self, &mut s);
        s
    }

    pub fn reverse(self) -> Self {
        match self {
            Reverse { rope } => *rope,
            Concatenation { left, right, .. } => concatenate(right.reverse(), left.reverse()),
            this => Reverse { rope: Box::new(this) },
        }
    }

    pub fn char_ref(&self, idx: usize) -> Option<&char> {
        match *self {
            Concatenation {
                ref left, ref right, ..
            } => {
                let llen = left.len();
                if idx < llen {
                    left.char_ref(idx)
                } else if idx < llen + right.len() {
                    right.char_ref(idx - llen)
                } else {
                    None
                }
            }
            FlatCharVec { ref seq } => {
                if idx < seq.len() {
                    Some(&seq[idx])
                } else {
                    None
                }
            }
            SubString {
                ref rope,
                ref offset,
                ref length,
            } => {
                if idx < *length {
                    rope.char_ref(offset + idx)
                } else {
                    None
                }
            }
            Reverse { ref rope } => {
                let len = rope.len();
                rope.char_ref(len - idx - 1)
            }
        }
    }

    pub fn char_ref_mut(&mut self, idx: usize) -> Option<&mut char> {
        match *self {
            Concatenation {
                ref mut left,
                ref mut right,
                ..
            } => {
                let llen = left.len();
                if idx < llen {
                    left.char_ref_mut(idx)
                } else if idx < llen + right.len() {
                    right.char_ref_mut(idx - llen)
                } else {
                    None
                }
            }
            FlatCharVec { ref mut seq } => {
                if idx < seq.len() {
                    Some(&mut seq[idx])
                } else {
                    None
                }
            }
            SubString {
                ref mut rope,
                ref offset,
                ref length,
            } => {
                if idx < *length {
                    rope.char_ref_mut(offset + idx)
                } else {
                    None
                }
            }
            Reverse { ref mut rope } => {
                let len = rope.len();
                rope.char_ref_mut(len - idx - 1)
            }
        }
    }

    // FIXME: clone?
    pub fn delete(self, start: usize, end: usize) -> Self {
        let tail = self.clone().slice_from(end);
        self.slice_to(start).append(tail)
    }

    pub fn slice_from(self, start: usize) -> Self {
        let len = self.len();
        self.slice(start, len)
    }

    pub fn slice_to(self, end: usize) -> Self {
        self.slice(0, end)
    }

    pub fn slice(self, start: usize, end: usize) -> Self {
        assert!(end <= self.len(), "illegal slice()");
        let slen = self.len();
        if start == 0 && end == slen {
            self
        } else {
            match self {
                Concatenation { left, right, .. } => {
                    let llen = left.len();
                    if end <= llen {
                        left.slice(start, end)
                    } else if start >= llen {
                        right.slice(start - llen, end - llen)
                    } else {
                        concatenate(left.slice(start, llen), right.slice(0, end - llen))
                    }
                }
                this @ FlatCharVec { .. } => {
                    if end - start < 16 {
                        if let FlatCharVec { seq } = this {
                            FlatCharVec {
                                seq: seq[start..end].to_vec(),
                            }
                        } else {
                            unreachable!()
                        }
                    } else {
                        SubString {
                            rope: Box::new(this),
                            offset: start,
                            length: end - start,
                        }
                    }
                }
                SubString { rope, offset, length } => {
                    assert!(end - start <= length);
                    SubString {
                        rope: rope,
                        offset: offset + start,
                        length: end - start,
                    }
                }
                Reverse { rope } => rope.slice(slen - end, slen - start).reverse(),
            }
        }
    }

    pub fn insert<T: IntoRope>(self, offset: usize, s: T) -> Self {
        assert!(offset <= self.len());
        let r = s.into_rope();
        if offset == 0 {
            r.append(self)
        } else if offset == self.len() {
            self.append(r)
        } else {
            self.clone().slice_to(offset).append(r).append(self.slice_from(offset))
        }
    }

    pub fn peek<F>(self, mut f: F) -> Self
    where
        F: FnMut(&Self),
    {
        f(&self);
        self
    }

    fn into_chars(self) -> Vec<char> {
        match self {
            FlatCharVec { seq } => seq,
            Concatenation { left, right, .. } => {
                let mut lchs = left.into_chars();
                let rchs = right.into_chars();
                lchs.extend(rchs);
                lchs
            }
            Reverse { rope } => {
                let mut inner = rope.into_chars();
                inner.reverse();
                inner
            }
            SubString { rope, offset, length } => {
                let inner = rope.into_chars();
                inner[offset..offset + length].to_vec()
            }
        }
    }
}

impl<'a> ::std::convert::From<&'a str> for Rope {
    fn from(s: &'a str) -> Rope {
        Rope::from_vec(s.chars().collect())
    }
}

impl ::std::convert::From<String> for Rope {
    fn from(s: String) -> Rope {
        Rope::from_vec(s.chars().collect())
    }
}

impl ops::Index<usize> for Rope {
    type Output = char;
    fn index(&self, index: usize) -> &Self::Output {
        self.char_ref(index).unwrap()
    }
}

// FIXME: Rope should not be a mutable structure
impl ops::IndexMut<usize> for Rope {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.char_ref_mut(index).unwrap()
    }
}

// TODO: how to implement this?
// impl ops::Index<ops::Range<usize>> for Rope {
// }

impl fmt::Display for Rope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for Rope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R{:?}", self.to_string())
    }
}

pub trait IntoRope {
    fn into_rope(self) -> Rope;
}

impl IntoRope for char {
    fn into_rope(self) -> Rope {
        FlatCharVec { seq: vec![self] }
    }
}

impl IntoRope for String {
    fn into_rope(self) -> Rope {
        FlatCharVec {
            seq: self.chars().collect(),
        }
    }
}

impl IntoRope for Rope {
    fn into_rope(self) -> Rope {
        self
    }
}

impl<'a> IntoRope for &'a str {
    fn into_rope(self) -> Rope {
        FlatCharVec {
            seq: self.chars().collect(),
        }
    }
}

#[test]
fn test_rope() {
    let s = Rope::from("Hello").append(' ').append("World").append('!');

    println!("depth => {}", s.depth());
    println!("len => {}", s.len());
    println!("got => {:?}", s);

    let s = s.insert(0, "Oh, man, ");

    println!("got => {:?}", s);

    let s = s.insert(9, "wait! ");
    println!("got => {:?}", s);
    println!("got => {:?}", s.reverse());
}

#[test]
fn test_rope2() {
    let s = Rope::from("Hello")
        .append(" abcdefghijklmnopqrstuvwxyz")
        .append(" World")
        .peek(|r| println!("D1 got => {}", r))
        .append(" abcdefghijklmnopqrstuvwxyz")
        .peek(|r| println!("D2 depth => {}", r.depth()))
        .append(" abcdefghijklmnopqrstuvwxyz")
        .append(" abcdefghijklmnopqrstuvwxyz");

    println!("depth => {}", s.depth());
    println!("len => {}", s.len());
    println!("got => {:?}", s);
    let s2 = s.slice(10, 50);
    println!("got => {:?}", s2);
    println!("len => {:?}", s2.len());
}

#[test]
fn test_rope_char_at() {
    // Note: if str to short, will be automaticlly flatten.
    let s = Rope::from("zzzzzzzzzzzzzzzzzzzzHello !").insert(26, "Worldxxxxxxxxxxxxxxxxxxxx");
    // case 0 flat
    assert_eq!(s.char_ref(20), Some(&'H'));
    // assert_eq!(s[20], 'H');
    // case 1 concat
    assert_eq!(s.char_ref(26), Some(&'W'));
    // will be "Held!"
    let s = s.delete(23, 30);
    assert_eq!(s.char_ref(23), Some(&'d'));
    // reverse
    let s = s.reverse().insert(30, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").delete(35, 40);
    for i in 0..s.len() {
        assert_eq!(
            s.to_string().chars().skip(i).next().unwrap(),
            s.char_ref(i).map(|&c| c).unwrap()
        );
    }
}

#[test]
fn test_rope_index_mut() {
    let mut s = Rope::from("zzzzzzzzzzzzzzzzzzzzHello !")
        .insert(23, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        .reverse()
        .delete(35, 40)
        .insert(26, "Worldxxxxxxxxxxxxxxxxxxxx");

    assert!(s[20] != 'd');
    s[20] = 'd';
    assert_eq!(s[20], 'd');
}

// TODO:
// pub trait Rope {
//     fn starts_with(self, prefix: &str) -> bool;
//     fn ends_with(self, suffix: &str) -> bool;
// }
