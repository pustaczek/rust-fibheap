#![feature(unsafe_block_in_unsafe_fn)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod embedlist;

use crate::embedlist::{EmbedList, EmbeddedListElem};
use std::ptr::NonNull;

pub struct FibHeap {
	trees: EmbedList<Node>,
	len: usize,
}

struct Node {
	parent: Option<NonNull<Node>>,
	children: EmbedList<Node>,
	left: NonNull<Node>,
	right: NonNull<Node>,
	key: i32,
	mark: bool,
	degree: usize,
}

impl FibHeap {
	pub fn new() -> FibHeap {
		FibHeap { trees: EmbedList::new(), len: 0 }
	}

	pub fn insert(&mut self, key: i32) {
		let node = Node::new(key);
		let is_smaller = self.trees.root().map_or(false, |root| root.key > key);
		self.trees.insert(node, is_smaller);
		self.len += 1;
	}
}

impl Drop for FibHeap {
	fn drop(&mut self) {
		fn drop_node(node: NonNull<Node>) {
			let mut node = unsafe { Box::from_raw(node.as_ptr()) };
			node.children.drop_custom(drop_node);
		}
		self.trees.drop_custom(drop_node);
	}
}

impl Node {
	fn new(key: i32) -> NonNull<Node> {
		let mut node = Box::new(Node {
			parent: None,
			children: EmbedList::new(),
			left: NonNull::dangling(),
			right: NonNull::dangling(),
			key,
			mark: false,
			degree: 0,
		});
		unsafe { node.embedlist_initalize() }
		NonNull::new(Box::into_raw(node)).unwrap()
	}
}

unsafe impl embedlist::EmbeddedListElem for Node {
	fn left_mut(&mut self) -> &mut NonNull<Self> {
		&mut self.left
	}

	fn right_mut(&mut self) -> &mut NonNull<Self> {
		&mut self.right
	}
}

#[test]
fn basic() {
	let mut heap = FibHeap::new();
	heap.insert(1);
	heap.insert(2);
	heap.insert(3);
}
