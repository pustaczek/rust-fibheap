#![feature(unsafe_block_in_unsafe_fn)]
#![forbid(unsafe_op_in_unsafe_fn)]

mod embedlist;

use crate::embedlist::IntrusiveList;
use std::ptr::NonNull;

pub struct FibHeap {
	trees: IntrusiveList<Node>,
	len: usize,
}

struct Node {
	parent: Option<NonNull<Node>>,
	children: IntrusiveList<Node>,
	left: NonNull<Node>,
	right: NonNull<Node>,
	key: i32,
	mark: bool,
	degree: usize,
}

impl FibHeap {
	pub fn new() -> FibHeap {
		FibHeap { trees: IntrusiveList::new(), len: 0 }
	}

	pub fn insert(&mut self, key: i32) {
		let node = Node::new(key);
		self.insert_node(node);
		self.len += 1;
	}

	fn insert_node(&mut self, mut node: NonNull<Node>) {
		let key = unsafe { node.as_mut() }.key;
		let is_smaller = self.trees.root().map_or(false, |root| root.key > key);
		self.trees.insert(node);
		if is_smaller {
			self.trees.set_root(node);
		}
	}

	pub fn pop_min(&mut self) -> Option<i32> {
		let mut root = unsafe { Box::from_raw(self.trees.extract_root()?.as_ptr()) };
		while let Some(kid) = root.children.extract_root() {
			self.trees.insert(kid);
		}
		self.consolidate();
		self.len -= 1;
		Some(root.key)
	}

	fn consolidate(&mut self) {
		let mut uniques: [Option<NonNull<Node>>; 10] = [None; 10];
		while let Some(mut node) = self.trees.extract_root() {
			let mut degree = unsafe { node.as_mut() }.degree;
			while let Some(mut other) = uniques[degree].take() {
				if unsafe { node.as_mut() }.key > unsafe { other.as_mut() }.key {
					std::mem::swap(&mut node, &mut other);
				}
				self.link(node, other);
				degree += 1;
			}
			uniques[degree] = Some(node);
		}
		for unique in &mut uniques {
			if let Some(node) = unique.take() {
				self.insert_node(node);
			}
		}
	}

	fn link(&mut self, mut upper: NonNull<Node>, mut lower: NonNull<Node>) {
		unsafe { lower.as_mut() }.parent = Some(upper);
		unsafe { lower.as_mut() }.mark = false;
		unsafe { upper.as_mut() }.degree += 1;
		unsafe { upper.as_mut() }.children.insert(lower);
	}

	pub fn merge(mut lhs: FibHeap, mut rhs: FibHeap) -> FibHeap {
		let new_min = match (lhs.trees.root(), rhs.trees.root()) {
			(Some(left), Some(right)) => Some(if left.key <= right.key { left } else { right }),
			(Some(only), None) | (None, Some(only)) => Some(only),
			(None, None) => None,
		};
		let new_min = new_min.map(NonNull::from);
		let len = lhs.len + rhs.len;
		lhs.trees.merge(std::mem::replace(&mut rhs.trees, IntrusiveList::new()));
		let mut trees = std::mem::replace(&mut lhs.trees, IntrusiveList::new());
		if let Some(new_min) = new_min {
			trees.set_root(new_min);
		}
		FibHeap { trees, len }
	}
}

impl Default for FibHeap {
	fn default() -> Self {
		FibHeap::new()
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
			children: IntrusiveList::new(),
			left: NonNull::dangling(),
			right: NonNull::dangling(),
			key,
			mark: false,
			degree: 0,
		});
		embedlist::initialize_elem(&mut *node);
		NonNull::from(Box::leak(node))
	}
}

unsafe impl embedlist::IntrusiveListElem for Node {
	fn left_mut(&mut self) -> &mut NonNull<Self> {
		&mut self.left
	}

	fn right_mut(&mut self) -> &mut NonNull<Self> {
		&mut self.right
	}
}

#[test]
fn single_element() {
	let mut heap = FibHeap::new();
	heap.insert(1);
	assert_eq!(heap.pop_min(), Some(1));
	assert_eq!(heap.pop_min(), None);
}

#[test]
fn two_elements() {
	let mut heap = FibHeap::new();
	heap.insert(2);
	heap.insert(1);
	assert_eq!(heap.pop_min(), Some(1));
	assert_eq!(heap.pop_min(), Some(2));
	assert_eq!(heap.pop_min(), None);
}

#[test]
fn insert_and_pop_four() {
	let mut heap = FibHeap::new();
	heap.insert(0);
	heap.insert(0);
	heap.insert(0);
	heap.insert(0);
	assert_eq!(heap.pop_min(), Some(0));
	assert_eq!(heap.pop_min(), Some(0));
	assert_eq!(heap.pop_min(), Some(0));
	assert_eq!(heap.pop_min(), Some(0));
	assert_eq!(heap.pop_min(), None);
}

#[test]
fn merge_interleaving() {
	let mut h1 = FibHeap::new();
	let mut h2 = FibHeap::new();
	h1.insert(1);
	h1.insert(3);
	h2.insert(2);
	h2.insert(4);
	let mut heap = FibHeap::merge(h1, h2);
	assert_eq!(heap.pop_min(), Some(1));
	assert_eq!(heap.pop_min(), Some(2));
	assert_eq!(heap.pop_min(), Some(3));
	assert_eq!(heap.pop_min(), Some(4));
	assert_eq!(heap.pop_min(), None);
}
