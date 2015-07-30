use std::mem;
use std::ptr;

struct Rawlink<T> {
    p: *mut T
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
    next: Option<Box<Node<T>>>
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

pub struct Queue<T> {
    first: Option<Box<Node<T>>>,
    last: Rawlink<Node<T>>
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue { first: None, last: Rawlink::none() }
    }

    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }

    pub fn enqueue(&mut self, val: T) {
        let ref old_last = self.last.take();
        let mut last = Box::new(Node {
            val: val,
            next: None
        });
        self.last = Rawlink::some(&mut last);
        if self.is_empty() {
            self.first = Some(last)
        } else {
            unsafe {
                (*old_last.p).next = Some(last)
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.first.is_none() {
            return None;
        }
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
