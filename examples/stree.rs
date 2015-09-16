extern crate adivon;

use adivon::suffix_tree::SuffixTree;


fn main() {
    let s = "".chars().collect::<Vec<char>>();
    let st = SuffixTree::new(&s);
    println!("{}", st.to_dot());

}
