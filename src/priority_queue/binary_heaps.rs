const INITIAL_SIZE: usize = 1;

/// Generic max priority queue implementation with a binary heap
pub struct MaxPQ<Key> {
    pq: Vec<Option<Key>>,
    n: usize,
}

impl<Key: PartialOrd> MaxPQ<Key> {
    fn with_capacity(capacity: usize) -> MaxPQ<Key> {
        let mut pq = Vec::with_capacity(capacity + 1);
        for _ in 0..capacity + 1 {
            pq.push(None);
        }
        MaxPQ { pq, n: 0 }
    }

    fn resize(&mut self, capacity: usize) {
        assert!(capacity > self.n);
        let mut temp = Vec::with_capacity(capacity);
        for i in 0..capacity {
            if i >= 1 && i <= self.n {
                temp.push(self.pq[i].take());
            } else {
                temp.push(None);
            }
        }
        self.pq = temp;
    }

    fn swim(&mut self, k: usize) {
        let mut k = k;
        while k > 1 && self.pq[k / 2] < self.pq[k] {
            self.pq.swap(k, k / 2);
            k /= 2;
        }
    }

    fn sink(&mut self, k: usize) {
        let mut k = k;
        while 2 * k <= self.n {
            let mut j = 2 * k;
            if j < self.n && self.pq[j] < self.pq[j + 1] {
                j += 1;
            }
            if self.pq[k] >= self.pq[j] {
                break;
            }
            self.pq.swap(k, j);
            k = j;
        }
    }
}

impl<Key: PartialOrd> Default for MaxPQ<Key> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key: PartialOrd> MaxPQ<Key> {
    /// create an empty priority queue
    pub fn new() -> Self {
        MaxPQ::with_capacity(INITIAL_SIZE)
    }

    /// create a priority queue with given keys
    pub fn from_vec(a: Vec<Key>) -> Self {
        let mut pq = Self::new();
        for i in a {
            pq.insert(i);
        }
        pq
    }

    /// insert a key into the priority queue
    pub fn insert(&mut self, x: Key) {
        let len = self.pq.len();
        if self.n == len - 1 {
            self.resize(2 * len);
        }
        self.n += 1;
        let n = self.n;
        self.pq[n] = Some(x);
        self.swim(n);
    }

    /// return and remove the largest key
    pub fn del_max(&mut self) -> Option<Key> {
        let max = self.pq[1].take();
        self.pq.swap(1, self.n);
        self.n -= 1;
        self.sink(1);
        let len = self.pq.len();
        if self.n > 0 && self.n == (len - 1) / 4 {
            self.resize(len / 2);
        }
        max
    }

    /// is the priority queue empty?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// return the largest key
    pub fn max(&self) -> Option<&Key> {
        self.pq[1].as_ref()
    }

    /// number of entries in the priority queue
    #[inline]
    pub fn size(&self) -> usize {
        self.n
    }
}

/// Generic min priority queue implementation with a binary heap
pub struct MinPQ<Key> {
    pq: Vec<Option<Key>>,
    n: usize,
}

impl<Key: PartialOrd> MinPQ<Key> {
    fn with_capacity(capacity: usize) -> MinPQ<Key> {
        let mut pq = Vec::with_capacity(capacity + 1);
        for _ in 0..capacity + 1 {
            pq.push(None);
        }
        MinPQ { pq, n: 0 }
    }

    fn resize(&mut self, capacity: usize) {
        assert!(capacity > self.n);
        let mut temp = Vec::with_capacity(capacity);
        for i in 0..capacity {
            if i >= 1 && i <= self.n {
                temp.push(self.pq[i].take());
            } else {
                temp.push(None);
            }
        }
        self.pq = temp;
    }

    fn swim(&mut self, k: usize) {
        let mut k = k;
        while k > 1 && self.pq[k / 2] > self.pq[k] {
            self.pq.swap(k, k / 2);
            k /= 2;
        }
    }

    fn sink(&mut self, k: usize) {
        let mut k = k;
        while 2 * k <= self.n {
            let mut j = 2 * k;
            if j < self.n && self.pq[j] > self.pq[j + 1] {
                j += 1;
            }
            if self.pq[k] <= self.pq[j] {
                break;
            }
            self.pq.swap(k, j);
            k = j;
        }
    }
}

impl<Key: PartialOrd> Default for MinPQ<Key> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key: PartialOrd> MinPQ<Key> {
    /// create an empty priority queue
    pub fn new() -> Self {
        MinPQ::with_capacity(INITIAL_SIZE)
    }

    /// create a priority queue with given keys
    pub fn from_vec(a: Vec<Key>) -> Self {
        let mut pq = Self::new();
        for i in a {
            pq.insert(i);
        }
        pq
    }

    /// insert a key into the priority queue
    pub fn insert(&mut self, x: Key) {
        let len = self.pq.len();
        if self.n == len - 1 {
            self.resize(2 * len);
        }
        self.n += 1;
        let n = self.n;
        self.pq[n] = Some(x);
        self.swim(n);
    }

    /// return and remove the smallest key
    pub fn del_min(&mut self) -> Option<Key> {
        let min = self.pq[1].take();
        self.pq.swap(1, self.n);
        self.n -= 1;
        self.sink(1);
        let len = self.pq.len();
        if self.n > 0 && self.n == (len - 1) / 4 {
            self.resize(len / 2);
        }
        min
    }

    /// is the priority queue empty?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// return the smallest key
    pub fn min(&self) -> Option<&Key> {
        self.pq[1].as_ref()
    }

    /// number of entries in the priority queue
    pub fn size(&self) -> usize {
        self.n
    }
}

#[test]
fn test_binary_heap_min_priority_queue() {
    let mut pq: MinPQ<char> = MinPQ::new();

    pq.insert('P');
    pq.insert('Q');
    pq.insert('E');

    assert_eq!(pq.size(), 3);
    assert_eq!(pq.del_min().unwrap(), 'E');
    assert_eq!(pq.size(), 2);

    pq.insert('X');
    pq.insert('A');
    pq.insert('M');

    assert_eq!(pq.del_min().unwrap(), 'A');

    pq.insert('P');
    pq.insert('L');
    pq.insert('B');

    assert_eq!(pq.del_min().unwrap(), 'B');

    assert_eq!(pq.size(), 6);
}

#[test]
fn test_binary_heap_max_priority_queue() {
    let mut pq: MaxPQ<char> = MaxPQ::new();

    pq.insert('P');
    pq.insert('Q');
    pq.insert('E');

    assert_eq!(pq.size(), 3);
    assert_eq!(pq.del_max().unwrap(), 'Q');
    assert_eq!(pq.size(), 2);

    pq.insert('X');
    pq.insert('A');
    pq.insert('M');

    assert_eq!(pq.del_max().unwrap(), 'X');

    pq.insert('P');
    pq.insert('L');
    pq.insert('E');

    assert_eq!(pq.del_max().unwrap(), 'P');

    assert_eq!(pq.size(), 6);
}

#[test]
fn test_empty_binary_heap() {
    let mut pq: MaxPQ<char> = MaxPQ::new();
    pq.insert('P');
    assert_eq!(pq.is_empty(), false);
    assert_eq!(pq.size(), 1);
    pq.del_max();
    assert_eq!(pq.size(), 0);
    assert_eq!(pq.is_empty(), true);
}
