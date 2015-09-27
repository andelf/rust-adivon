#![allow(unused_features)]
#![feature(iter_arith, str_char, append)]

extern crate vec_map;
extern crate rand;


pub mod bag;
pub mod stack;
pub mod queue;
pub mod deque;
pub mod graph;
pub mod priority_queue;
pub mod hashst;
pub mod tries;

pub mod suffix_tree;
pub mod splay_tree;

pub mod rope;

pub mod skip_list;

pub mod prelude;
pub use prelude::*;
