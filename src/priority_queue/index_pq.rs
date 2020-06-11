use std::iter;
use std::usize;

pub struct IndexMinPQ<T: PartialOrd> {
    nmax: usize,
    n: usize,
    pq: Vec<usize>,
    qp: Vec<usize>,
    keys: Vec<Option<T>>,
}

impl<T: PartialOrd> IndexMinPQ<T> {
    pub fn with_capacity(nmax: usize) -> IndexMinPQ<T> {
        let mut keys = Vec::new();
        for _ in 0..nmax + 1 {
            keys.push(None);
        }
        IndexMinPQ {
            nmax: nmax,
            n: 0,
            pq: iter::repeat(0).take(nmax + 1).collect(),
            qp: iter::repeat(usize::MAX).take(nmax + 1).collect(),
            keys: keys,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn contains(&self, i: usize) -> bool {
        assert!(i < self.nmax, "index out of bounds");
        self.qp[i] != usize::MAX
    }

    pub fn size(&self) -> usize {
        self.n
    }

    // Associates key with index i
    pub fn insert(&mut self, i: usize, key: T) {
        assert!(i < self.nmax, "index out of bounds");
        if self.contains(i) {
            panic!("index already in pq");
        }
        self.n += 1;
        self.qp[i] = self.n;
        self.pq[self.n] = i;
        self.keys[i] = Some(key);
        let n = self.n;
        self.swim(n)
    }

    pub fn min_index(&self) -> usize {
        assert!(self.n != 0, "priority queue underflow");
        self.pq[1]
    }

    pub fn min_key(&self) -> Option<&T> {
        if self.n == 0 {
            None
        } else {
            self.keys[self.pq[1]].as_ref()
        }
    }

    pub fn del_min(&mut self) -> Option<usize> {
        if self.n == 0 {
            None
        } else {
            let min = self.pq[1];
            let n = self.n;
            self.exch(1, n);
            self.n -= 1;
            self.sink(1);
            self.qp[min] = usize::MAX; // delete
                                       // help with gc
            self.keys[self.pq[self.n + 1]] = None;
            self.pq[self.n + 1] = usize::MAX;
            Some(min)
        }
    }

    pub fn key_of(&self, i: usize) -> Option<&T> {
        if i >= self.nmax || !self.contains(i) {
            None
        } else {
            self.keys[i].as_ref()
        }
    }

    pub fn change_key(&mut self, i: usize, key: T) {
        if i >= self.nmax || !self.contains(i) {
            panic!("blah....");
        }
        self.keys[i] = Some(key);
        let p = self.qp[i];
        self.swim(p);
        let p = self.qp[i];
        self.sink(p);
    }

    pub fn decrease_key(&mut self, i: usize, key: T) {
        if i >= self.nmax || !self.contains(i) {
            panic!("decrease_key");
        }
        self.keys[i] = Some(key);
        let p = self.qp[i];
        self.swim(p);
    }

    pub fn increase_key(&mut self, i: usize, key: T) {
        if i >= self.nmax || !self.contains(i) {
            panic!("increase_key");
        }
        self.keys[i] = Some(key);
        let p = self.qp[i];
        self.sink(p);
    }

    pub fn delete(&mut self, i: usize) {
        if i >= self.nmax || !self.contains(i) {
            panic!("delete");
        }
        let index = self.qp[i];
        let n = self.n;
        self.exch(index, n);
        self.n -= 1;
        self.swim(index);
        self.sink(index);
        self.keys[i] = None;
        self.qp[i] = usize::MAX;
    }

    #[inline]
    fn greater(&self, i: usize, j: usize) -> bool {
        self.keys[self.pq[i]] > self.keys[self.pq[j]]
    }

    fn exch(&mut self, i: usize, j: usize) {
        self.pq.swap(i, j);
        self.qp.swap(self.pq[i], self.pq[j]);
    }

    fn swim(&mut self, k: usize) {
        let mut k = k;
        while k > 1 && self.greater(k / 2, k) {
            self.exch(k, k / 2);
            k /= 2;
        }
    }

    fn sink(&mut self, k: usize) {
        let mut k = k;
        while 2 * k <= self.n {
            let mut j = 2 * k;
            if j < self.n && self.greater(j, j + 1) {
                j += 1;
            }
            if !self.greater(k, j) {
                break;
            }
            self.exch(k, j);
            k = j;
        }
    }
}

#[test]
fn test_index_min_pq() {
    let strings = vec!["it", "was", "the", "best", "of", "times", "it", "was", "the", "worst"];
    let mut pq = IndexMinPQ::with_capacity(strings.len());

    for (i, s) in strings.iter().enumerate() {
        pq.insert(i, s);
    }

    while !pq.is_empty() {
        let i = pq.del_min().unwrap();
        assert!(!strings[i].is_empty());
    }

    for (i, s) in strings.iter().enumerate() {
        pq.insert(i, s);
    }

    while !pq.is_empty() {
        pq.del_min();
    }
}
