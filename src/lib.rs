#![feature(associated_type_defaults, plugin)]
#![allow(mutable_transmutes)]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![cfg_attr(not(feature = "dev"), allow(unknown_lints))]

pub mod bag;
pub mod deque;
pub mod graph;
pub mod hashst;
pub mod priority_queue;
pub mod queue;
pub mod stack;
pub mod tries;

pub mod splay_tree;
pub mod suffix_tree;

pub mod rope;

pub mod skip_list;

pub mod rbtree;

pub mod kdtree;
pub mod primitive;

pub mod union_find;

pub mod prelude;
pub use prelude::*;
