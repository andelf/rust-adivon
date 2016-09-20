#![feature(associated_type_defaults, plugin)]
#![allow(mutable_transmutes)]

#![cfg_attr(test, feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![cfg_attr(not(feature = "dev"), allow(unknown_lints))]

extern crate vec_map;
extern crate rand;

#[cfg(test)]
extern crate quickcheck;

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

pub mod rbtree;

pub mod primitive;
pub mod kdtree;

pub mod union_find;

pub mod prelude;
pub use prelude::*;
