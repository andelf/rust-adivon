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
    item: T,
    next: Option<Box<Node<T>>>
}

impl<T> Node<T> {
    /// work around for moved value
    fn into_item_and_next(self) -> (T, Option<Box<Node<T>>>) {
        (self.item, self.next)
    }

    fn size(&self) -> usize {
        match self.next {
            Some(ref n) => 1 + n.size(),
            None        => 1
        }
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

    pub fn enqueue(&mut self, item: T) {
        let ref old_last = self.last.take();
        let mut last = Box::new(Node {
            item: item,
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

    pub fn dequeue(&mut self) -> T {
        let old_first = self.first.take();
        let (item, first) = old_first.unwrap().into_item_and_next();
        self.first = first;
        if self.is_empty() {
            self.last = Rawlink::none()
        }
        item
    }

    pub fn size(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.first.as_ref().unwrap().size()
        }
    }
}


#[test]
fn test_linked_queue() {
    let mut queue = Queue::<String>::new();

    assert!(queue.is_empty());
    let mut result = "to be or not to be".split(' ');

    for s in "to be or not to - be - - that - - - is".split(' ') {
        if s == "-" {
            assert_eq!(&queue.dequeue(), result.next().unwrap())
        } else {
            queue.enqueue(s.into())
        }
    }
    assert!(!queue.is_empty());
    assert_eq!(2, queue.size());
}
