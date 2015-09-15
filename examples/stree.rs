extern crate adivon;

use adivon::suffix_tree::SuffixTree;


fn main() {
    let s = "abcabxabcdaabab".chars().collect::<Vec<char>>();
    let st = SuffixTree::new(&s);
    println!("{}", st.to_dot());

    println!("fuuuuck {}", st.contains(&"bxabcdaa".chars().collect::<Vec<char>>()));

    println!("contains {}", st.contains(&"abc".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"abcabxabcdaabab".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"cabxabcda".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"abcdaab".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"c".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"abxabc".chars().collect::<Vec<char>>()));
    println!("contains {}", st.contains(&"aabab".chars().collect::<Vec<char>>()));
}
