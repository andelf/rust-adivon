use std::fmt;

pub struct UnionFind {
    id: Vec<usize>,
    /// number of objects in the tree rooted at i.
    rank: Vec<usize>,
    count: usize
}

impl UnionFind {
    pub fn new(n: usize) -> UnionFind {
        UnionFind {
            id: (0..n).collect(),
            rank: vec![0; n],
            count: n
        }
    }

    // root_of
    pub fn find(&mut self, mut p: usize) -> usize {
        assert!(p < self.id.len());
        while p != self.id[p] {
            self.id[p] = self.id[self.id[p]];    // path compression by halving
            p = self.id[p];
        }
        p
    }

    pub fn count(&self) -> usize {
        self.count
    }

    /// Are the two sites p and q in the same component?
    pub fn connected(&mut self, p: usize, q: usize) -> bool {
        self.find(p) == self.find(q)
    }

    pub fn union(&mut self, p: usize, q: usize) {
        let i = self.find(p);
        let j = self.find(q);

        if i == j {
            return;
        }
        if self.rank[i] < self.rank[j] {
            self.id[i] = j;
        } else if self.rank[i] > self.rank[j] {
            self.id[j] = i;
        } else {
            self.id[j] = i;
            self.rank[i] += 1;
        }
        self.count -= 1;
    }
}

impl fmt::Display for UnionFind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in self.id.iter() {
            try!(write!(f, "{} ", i));
        }
        Ok(())
    }
}


#[test]
fn test_uf() {
    let mut uf = UnionFind::new(10);
    uf.union(4, 3);
    uf.union(3, 8);
    uf.union(6, 5);
    uf.union(9, 4);
    uf.union(2, 1);
    uf.union(5, 0);
    uf.union(7, 2);
    uf.union(6, 1);
    assert!(uf.count() == 2);
}
