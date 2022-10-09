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
    val: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    /// work around for moved value
    fn into_val_and_next(self) -> (T, Option<Box<Node<T>>>) {
        (self.val, self.next)
    }

    fn len(&self) -> usize {
        self.next.as_ref().map_or(1, |n| 1 + n.len())
    }
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            val: self.val.clone(),
            next: self.next.clone(),
        }
    }
}

pub struct Queue<T> {
    first: Option<Box<Node<T>>>,
    last: Rawlink<Node<T>>,
}

impl<T: Clone> Clone for Queue<T> {
    fn clone(&self) -> Self {
        let mut first = self.first.clone();
        let last = {
            let mut p = first.as_mut();
            while p.as_ref().map_or(false, |n| n.next.is_some()) {
                p = p.unwrap().next.as_mut();
            }
            p.map_or_else(Rawlink::none, |n| Rawlink::some(&mut **n))
        };
        Queue { first, last }
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue {
            first: None,
            last: Rawlink::none(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }

    pub fn enqueue(&mut self, val: T) {
        let old_last = &self.last.take();
        let mut last = Box::new(Node { val, next: None });
        self.last = Rawlink::some(&mut last);
        if self.is_empty() {
            self.first = Some(last)
        } else {
            unsafe { (*old_last.p).next = Some(last) }
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.first.as_ref()?;
        let old_first = self.first.take();
        let (val, first) = old_first.unwrap().into_val_and_next();
        self.first = first;
        if self.first.is_none() {
            self.last = Rawlink::none()
        }
        Some(val)
    }

    pub fn peek(&self) -> Option<&T> {
        self.first.as_ref().map(|n| &(*n).val)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.first.as_mut().map(|n| &mut (*n).val)
    }

    pub fn len(&self) -> usize {
        self.first.as_ref().map_or(0, |n| n.len())
    }
}

pub struct IntoIter<T> {
    queue: Queue<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.queue.dequeue()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.queue.len();
        (length, Some(length))
    }
}

impl<T> IntoIterator for Queue<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { queue: self }
    }
}

pub struct Iter<'a, T>
where
    T: 'a,
{
    node: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let ret = self.node.map(|n| &n.val);
        self.node = self.node.and_then(|n| n.next.as_deref());
        ret
    }

    // Bad
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut sz = 0;
        let mut p = self.node;
        while p.is_some() {
            p = p.and_then(|n| n.next.as_deref());
            sz += 1;
        }
        (sz, Some(sz))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<T> Queue<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            node: self.first.as_deref(),
        }
    }
}

#[test]
fn test_queue() {
    let mut queue = Queue::<&str>::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    assert_eq!(queue.peek(), None);

    queue.enqueue("welcome");
    queue.enqueue("to");
    queue.enqueue("china");
    assert!(!queue.is_empty());
    assert_eq!(queue.len(), 3);

    assert_eq!(queue.peek(), Some(&"welcome"));
    queue.peek_mut().map(|val| *val = "go");
    assert_eq!(queue.peek(), Some(&"go"));
}

#[test]
fn test_queue_clone() {
    let mut queue1 = Queue::<&str>::new();
    queue1.enqueue("welcome");
    queue1.enqueue("to");
    queue1.enqueue("china");

    let mut queue2 = queue1.clone();
    queue2.dequeue();
    assert_eq!(queue1.peek(), Some(&"welcome"));
    assert_eq!(queue2.peek(), Some(&"to"));

    queue2.dequeue();
    queue2.dequeue();
    queue2.enqueue("beijing");

    assert_eq!(queue1.peek(), Some(&"welcome"));
    assert_eq!(queue2.peek(), Some(&"beijing"));
}

#[test]
fn test_queue_into_iter() {
    let mut queue = Queue::<&str>::new();
    queue.enqueue("welcome");
    queue.enqueue("to");
    queue.enqueue("china");

    let result = vec!["welcome", "to", "china"];
    for i in queue {
        assert!(result.contains(&i));
    }
}

#[test]
fn test_queue_iter() {
    let mut queue = Queue::<&str>::new();
    queue.enqueue("welcome");
    queue.enqueue("to");
    queue.enqueue("china");

    let mut rit = vec!["welcome", "to", "china"].into_iter();
    for i in queue.iter() {
        assert_eq!(i, &rit.next().unwrap())
    }
}
