pub struct Node<T> {
    val: T,
    next: Option<Box<Node<T>>>,
}

pub struct Stack<T> {
    s: Option<Box<Node<T>>>
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { s: None }
    }

    pub fn push(&mut self, val: T) {
        let next = self.s.take();
        self.s = Some(Box::new(Node { val: val, next: next }))
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut top = self.s.take();
        self.s = top.as_mut().map(|t| t.next.take()).unwrap_or(None);
        top.map(|n| n.val)
    }

    pub fn len(&mut self) -> usize {
        let mut sz = 0;
        let mut p = self.s.as_ref();
        while p.is_some() {
            p = p.map(|n| n.next.as_ref()).unwrap_or(None);
            sz += 1;
        }
        sz
    }

    pub fn peek(&self) -> Option<&T> {
        self.s.as_ref().map(|n| &n.val)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.s.as_mut().map(|n| &mut n.val)
    }
}



#[test]
fn test_stack() {
    let mut s = Stack::new();
    assert_eq!(s.len(), 0);
    s.push(1000);
    assert_eq!(s.len(), 1);
    s.push(2000);
    assert_eq!(s.len(), 2);
    assert_eq!(s.peek(), Some(&2000));
    assert_eq!(s.pop(), Some(2000));
    assert_eq!(s.len(), 1);
    assert_eq!(s.pop(), Some(1000));
    assert_eq!(s.len(), 0);
    assert_eq!(s.pop(), None);
    assert_eq!(s.len(), 0);

    s.push(250);
    s.peek_mut().map(|val| *val = 100);
    assert_eq!(s.pop(), Some(100));
}
