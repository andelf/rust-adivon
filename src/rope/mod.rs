use std::fmt;

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
    FlatCharVec { seq: Vec<char> },
    Concatenation { left: Box<Rope>, right: Box<Rope>, depth: usize, length: usize },
    Reverse { rope: Box<Rope> },
    SubString { rope: Box<Rope>, offset: usize, length: usize },
}

fn write_rope_to_string(rope: &Rope, s: &mut String) {
    match *rope {
        FlatCharVec { ref seq }                   => {
            s.push_str(&seq.iter().map(Clone::clone).collect::<String>());
        }
        Concatenation { ref left, ref right, .. } => {
            write_rope_to_string(left, s);
            write_rope_to_string(right, s);
        },
        Reverse { ref rope }                      => {
            let inner = rope.to_string();
            s.push_str(&inner.chars().rev().collect::<String>());
        }
        SubString { ref rope, offset, length }    => {
            let inner = rope.to_string();
            s.push_str(&inner[offset..offset+length]);
        }
    }
}

fn concatenate(left: Rope, right: Rope) -> Rope {
    if left.len() == 0 {
        return right;
    }
    if right.len() == 0 {
        return left;
    }

    const COMBINE_LENGTH: usize = 17;
    let length = left.len() + right.len();
    if length < COMBINE_LENGTH {
        let mut lchs = left.into_chars();
        lchs.extend(right.into_chars());
        return FlatCharVec { seq: lchs };
    }
    let depth = max(left.depth(), right.depth()) + 1;
    Concatenation { left: Box::new(left), right: Box::new(right), depth: depth, length: length }
}


impl Rope {
    pub fn from_str(s: &str) -> Rope {
        Rope::from_vec(s.chars().collect())
    }

    pub fn from_vec(seq: Vec<char>) -> Rope {
        Rope::FlatCharVec { seq: seq }
    }

    pub fn len(&self) -> usize {
        match *self {
            FlatCharVec { ref seq }      => seq.len(),
            Concatenation { length, .. } => length,
            Reverse { ref rope }         => rope.len(),
            SubString { length, .. }     => length
        }
    }

    pub fn depth(&self) -> usize {
        match *self {
            FlatCharVec { .. }          => 0,
            Concatenation { depth, .. } => depth,
            Reverse { ref rope }        => rope.depth(),
            SubString { ref rope, .. }  => rope.depth()
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
            Concatenation { left, right, .. } => {
                concatenate(right.reverse(), left.reverse())
            },
            this => Reverse { rope: Box::new(this) }
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
                        right.slice(start-llen, end-llen)
                    } else  {
                        concatenate(left.slice(start, llen), right.slice(0, end-llen))
                    }
                },
                this@FlatCharVec { .. } => {
                    if end - start < 16 {
                        if let FlatCharVec { seq } = this {
                            FlatCharVec { seq: seq[start..end].to_vec() }
                        } else {
                            unreachable!()
                        }
                    } else {
                        SubString { rope: Box::new(this), offset: start, length: end-start }
                    }
                },
                SubString { rope, offset, length }  => {
                    assert!(end - start <= length);
                    SubString { rope: rope, offset: offset + start, length: end - start }
                },
                Reverse { rope } => {
                    rope.slice(slen-end, slen-start).reverse()
                }
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

    fn into_chars(self) -> Vec<char> {
        match self {
            FlatCharVec { seq }         => seq,
            Concatenation { left, right, .. } => {
                let mut lchs = left.into_chars();
                let rchs = right.into_chars();
                lchs.extend(rchs);
                lchs
            }
            Reverse { rope }        => {
                let mut inner = rope.into_chars();
                inner.reverse();
                inner
            }
            SubString { rope, offset, length }  => {
                let inner = rope.into_chars();
                inner[offset .. offset+length].to_vec()
            }
        }
    }

    fn is_balanced(&self) -> bool {
        true
    }
}

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
        FlatCharVec { seq: self.chars().collect() }
    }
}

impl IntoRope for Rope {
    fn into_rope(self) -> Rope {
        self
    }
}

impl<'a> IntoRope for &'a str {
    fn into_rope(self) -> Rope {
        FlatCharVec { seq: self.chars().collect() }
    }
}

#[test]
fn test_rope() {
    let s = Rope::from_str("Hello").append(' ').append("World").append('!');

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
    let s = Rope::from_str("Hello")
        .append(" abcdefghijklmnopqrstuvwxyz")
        .append(" World")
        .append(" abcdefghijklmnopqrstuvwxyz")
        .append(" abcdefghijklmnopqrstuvwxyz")
        .append(" abcdefghijklmnopqrstuvwxyz");

    println!("depth => {}", s.depth());
    println!("len => {}", s.len());
    println!("got => {:?}", s);
    let s2 = s.slice(10, 50);
    println!("got => {:?}", s2);
    println!("len => {:?}", s2.len());
}

// pub trait Rope {
//     fn find(&self, ch: char) -> Option<usize>;
//     // fn find(&self, pat: &str) -> Option<usize>;
//     fn insert(self, offset: usize, s: &str) -> Self;

//     fn rebalance(self) -> Self;

//     fn trim_left(self) -> Self;
//     fn trim_right(self) -> Self;
//     fn trim(self) -> Self {
//         self.trim_left().trim_right()
//     }

//     fn starts_with(self, prefix: &str) -> bool;
//     fn ends_with(self, suffix: &str) -> bool;
// }
