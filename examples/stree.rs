extern crate adivon;

use adivon::suffix_tree::SuffixTree;


fn main() {
    let para = "she sells sea shells on the sea shore she sells sea shells on the sea shorethe shells";
    let segs = para.split(' ').collect::<Vec<&str>>();
    let st = SuffixTree::new(&segs);
    println!("{}", st.to_dot());
}
