pub struct Node<T> {
    val: T,
    next: Option<Box<Node<T>>>,
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            val: self.val.clone(),
            next: self.next.clone()
        }
    }
}

pub struct Stack<T> {
    s: Option<Box<Node<T>>>
}

impl<T: Clone> Clone for Stack<T> {
    fn clone(&self) -> Self {
        Stack { s: self.s.clone() }
    }
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
        self.s = top.as_mut().map_or(None, |t| t.next.take());
        top.map(|n| n.val)
    }

    pub fn len(&self) -> usize {
        let mut sz = 0;
        let mut p = self.s.as_ref();
        while p.is_some() {
            p = p.map_or(None, |n| n.next.as_ref());
            sz += 1;
        }
        sz
    }

    pub fn is_empty(&self) -> bool {
        self.peek().is_none()
    }

    pub fn peek(&self) -> Option<&T> {
        self.s.as_ref().map(|n| &n.val)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.s.as_mut().map(|n| &mut n.val)
    }
}


pub struct IntoIter<T> {
    stack: Stack<T>
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.stack.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.stack.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.stack.len()
    }
}

impl<T> IntoIterator for Stack<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { stack: self }
    }
}

pub struct Iter<'a, T> where T: 'a {
    node: Option<&'a Box<Node<T>>>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let ret = self.node.map(|n| &n.val);
        self.node = self.node.map_or(None, |n| n.next.as_ref());
        ret
    }

    // Bad
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut sz = 0;
        let mut p = self.node;
        while p.is_some() {
            p = p.map_or(None, |n| n.next.as_ref());
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

impl<T> Stack<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            node: self.s.as_ref()
        }
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


#[test]
fn test_stack_into_iter() {
    let mut s = Stack::new();
    s.push(100);
    s.push(200);
    s.push(300);

    let mut result = vec![300, 200, 100].into_iter();
    for i in s {
        assert_eq!(i, result.next().unwrap());
    }
}


#[test]
fn test_stack_iter() {
    let mut s = Stack::new();
    s.push(100);
    s.push(200);
    s.push(300);

    let mut result = vec![300, 200, 100].into_iter();
    for i in s.iter() {
        assert_eq!(*i, result.next().unwrap());
    }

    assert_eq!(s.len(), 3);
}

#[test]
fn test_stack_clone() {
    let mut s = Stack::new();
    s.push(100);
    s.push(200);
    s.push(300);

    let t = s.clone();

    s.pop();
    assert_eq!(s.len(), 2);
    assert_eq!(t.len(), 3);
}
