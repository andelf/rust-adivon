use super::bag::Bag;

pub struct Graph {
    v: usize,
    e: usize,
    adj: Vec<Bag<usize>>
}

impl Graph {
    pub fn new(v: usize) -> Graph {
        let mut adj = Vec::with_capacity(v);
        for _ in 0 .. v {
            adj.push(Bag::new());
        }
        Graph {
            v: v,
            e: 0,
            adj: adj
        }
    }

    fn validate_vertex(&self, v: usize) {
        assert!(v < self.v, "vertex is not between 0 and {}", self.v - 1)
    }

    pub fn add_edge(&mut self, v: usize, w: usize) {
        self.validate_vertex(v);
        self.validate_vertex(w);

        self.e += 1;
        self.adj[v].add(w);
        self.adj[w].add(v);
    }

    pub fn to_dot(&self) -> String {
        let mut dot = String::new();

        dot.push_str("digraph G {\n");
        for i in 0 .. self.v {
            dot.push_str(&format!("  {};\n", i));
        }

        for (v, adj) in self.adj.iter().enumerate() {
            for w in adj.iter() {
                dot.push_str(&format!("  {} -> {};\n", v, w));
            }
        }
        dot.push_str("}\n");

        dot
    }
}


#[test]
fn test_graph() {
    let mut g = Graph::new(10);
    g.add_edge(0, 3);
    g.add_edge(0, 5);
    g.add_edge(4, 5);

    println!("got => \n{}", g.to_dot())
}
