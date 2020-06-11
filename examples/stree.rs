extern crate adivon;

use adivon::suffix_tree::SuffixTree;

fn main() {
    let s = "apple".chars().collect::<Vec<char>>();
    let s2 = "apple_tree".chars().collect::<Vec<char>>();
    let mut st = SuffixTree::new(&s);
    st.add(&s2);
    println!("{}", st.to_dot());
}
