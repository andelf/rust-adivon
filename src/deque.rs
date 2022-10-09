use std::fmt;
use std::mem;
use std::ptr;

struct Rawlink<T> {
    p: *mut T,
}

impl<T> Rawlink<T> {
    fn none() -> Rawlink<T> {
        Rawlink { p: ptr::null_mut() }
    }

    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink { p: n }
    }

    fn take(&mut self) -> Rawlink<T> {
        mem::replace(self, Rawlink::none())
    }
}

struct Node<T> {
    item: T,
    next: Option<Box<Node<T>>>,
    prev: Rawlink<Node<T>>,
}

impl<T> Node<T> {
    fn size(&self) -> usize {
        let mut p = self.next.as_ref();
        let mut sz = 1;
        while p.is_some() {
            p = p.unwrap().next.as_ref();
            sz += 1;
        }
        sz
    }
}

/// linked double queue
pub struct Deque<T> {
    first: Option<Box<Node<T>>>,
    last: Rawlink<Node<T>>,
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deque<T> {
    pub fn new() -> Deque<T> {
        Deque {
            first: None,
            last: Rawlink::none(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }

    pub fn len(&self) -> usize {
        self.first.as_ref().map_or(0, |n| n.size())
    }

    pub fn add_first(&mut self, item: T) {
        let mut old_first = self.first.take();
        let mut first = Box::new(Node {
            item,
            next: None,
            prev: Rawlink::none(),
        });

        if old_first.is_some() {
            old_first.as_mut().unwrap().prev = Rawlink::some(&mut first);
            first.next = old_first;
        } else {
            self.last = Rawlink::some(&mut first);
        }

        self.first = Some(first)
    }

    pub fn add_last(&mut self, item: T) {
        if self.first.is_some() {
            let old_last = self.last.take();
            let mut last = Box::new(Node {
                item,
                next: None,
                prev: Rawlink::none(),
            });
            self.last = Rawlink::some(&mut last);
            unsafe {
                (*old_last.p).next = Some(last);
            }
        } else {
            self.add_first(item)
        }
    }

    pub fn remove_first(&mut self) -> Option<T> {
        let old_first = self.first.take();
        if let Some(old_first) = old_first {
            let Node {
                item, next: mut first, ..
            } = *old_first;
            // update new first's prev field
            if let Some(v) = first.as_mut() {
                v.prev = Rawlink::none();
            }
            self.first = first;
            Some(item)
        } else {
            None
        }
    }

    pub fn remove_last(&mut self) -> Option<T> {
        let old_last = self.last.take();
        if old_last.p.is_null() {
            return None;
        }
        let last_ref_mut = unsafe { &mut *old_last.p };

        let last: Node<T> = mem::replace(last_ref_mut, unsafe { mem::zeroed() });

        if last.prev.p.is_null() {
            self.first = None;
        } else {
            unsafe {
                (*last.prev.p).next = None;
            }
        }
        self.last = last.prev;

        Some(last.item)
    }

    pub fn peek_first(&self) -> Option<&T> {
        self.first.as_ref().map(|n| &n.item)
    }

    pub fn peek_last(&self) -> Option<&T> {
        if self.last.p.is_null() {
            None
        } else {
            let last_ref = unsafe { &mut *self.last.p };
            Some(&last_ref.item)
        }
    }
}

impl<T> Deque<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            current: self.first.as_deref(),
            nelem: self.len(),
        }
    }
}

impl<T: fmt::Display> fmt::Display for Deque<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.first {
            None => {
                write!(f, "<empty deque>")?;
            }
            Some(ref l) => {
                write!(f, "(")?;
                let mut p = Some(l);
                while p.is_some() {
                    write!(f, "{},", p.unwrap().item)?;
                    p = p.unwrap().next.as_ref();
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

pub struct IntoIter<T> {
    q: Deque<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.q.remove_first()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.q.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.q.len()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.q.remove_last()
    }
}

impl<T> IntoIterator for Deque<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { q: self }
    }
}

// TODO impl DoubleEndedIterator
pub struct Iter<'a, T: 'a> {
    current: Option<&'a Node<T>>,
    nelem: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.nelem == 0 {
            return None;
        }
        let old_current = self.current.take();

        self.current = (*old_current.unwrap()).next.as_deref();
        self.nelem -= 1;
        Some(&old_current.as_ref().unwrap().item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.nelem, Some(self.nelem))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.nelem
    }
}

#[test]
fn test_linked_deque_add_remove() {
    let mut deque: Deque<i32> = Deque::new();

    assert!(deque.is_empty());
    assert_eq!(deque.remove_first(), None);
    assert_eq!(deque.remove_last(), None);

    let result = vec![4, 0, 5, 2, 3];
    let mut rit = result.iter();
    // -1 remove last
    // -2 remove first
    // extra 2 more -1 -2 will result None
    for s in vec![4, 2, 3, 0, -1, -2, 5, -2, -1, -1, -2] {
        if s == -2 {
            assert_eq!(deque.remove_first(), rit.next().map(|&v| v));
        } else if s == -1 {
            assert_eq!(deque.remove_last(), rit.next().map(|&v| v));
        } else {
            deque.add_first(s);
        }
    }

    assert!(deque.is_empty());
}

#[test]
fn test_linked_deque_size() {
    let mut deque: Deque<i32> = Deque::new();

    assert!(deque.is_empty());

    let result = vec![0, 1, 2, 3, 4, 3, 2, 3, 2, 1, 0, 0];
    let mut rit = result.iter();
    // -1 remove last
    // -2 remove first
    for s in vec![4, 2, 3, 0, -1, -2, 5, -1, -1, -2] {
        if s == -2 {
            assert_eq!(deque.len(), *rit.next().unwrap());
            deque.remove_first();
        } else if s == -1 {
            assert_eq!(deque.len(), *rit.next().unwrap());
            deque.remove_last();
        } else {
            assert_eq!(deque.len(), *rit.next().unwrap());
            deque.add_first(s);
        }
    }

    assert!(deque.is_empty());
}

#[test]
fn test_linked_deque_iter() {
    let mut deque: Deque<i32> = Deque::new();

    assert!(deque.is_empty());
    for i in 0..10 {
        if i % 2 == 0 {
            deque.add_first(i);
        } else {
            deque.add_last(i);
        }
    }

    let mut n = 0i32;
    let it = deque.iter();
    assert_eq!(it.len(), 10);

    for _ in it {
        n += 1;
    }
    assert_eq!(n, 10);
}

#[test]
fn test_deque_into_iter() {
    let mut deque: Deque<i32> = Deque::new();
    deque.add_last(12);
    deque.add_last(11);
    deque.add_last(10);
    deque.add_first(0);
    deque.add_first(5);
    deque.add_first(7);

    let mut rit = vec![7, 5, 0, 12, 11, 10].into_iter();
    for i in deque {
        assert_eq!(i, rit.next().unwrap())
    }
}

#[test]
fn test_deque_peek() {
    let mut deque: Deque<i32> = Deque::new();
    deque.add_last(12);
    deque.add_last(10);

    assert_eq!(deque.peek_last(), Some(&10));
    assert_eq!(deque.peek_first(), Some(&12));
    deque.add_last(11);
    deque.add_first(34);
    assert_eq!(deque.peek_last(), Some(&11));
    assert_eq!(deque.peek_first(), Some(&34));
}
