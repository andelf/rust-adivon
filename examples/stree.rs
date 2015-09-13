extern crate adivon;

use adivon::suffix_tree::SuffixTree;


fn main() {
    let st = SuffixTree::new(b"apple");
    println!("{}", st.to_dot());
}
