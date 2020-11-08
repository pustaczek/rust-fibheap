use std::ptr::NonNull;

pub unsafe trait IntrusiveListElem {
	fn left_mut(&mut self) -> &mut NonNull<Self>;
	fn right_mut(&mut self) -> &mut NonNull<Self>;
}

pub struct IntrusiveList<T> {
	root: Option<NonNull<T>>,
}

// TODO: This module is wildly unsafe (and not marked as such), try to figure out a better API.

impl<T: IntrusiveListElem> IntrusiveList<T> {
	pub fn new() -> Self {
		IntrusiveList { root: None }
	}

	fn singleton(node: NonNull<T>) -> Self {
		IntrusiveList { root: Some(node) }
	}

	pub fn insert(&mut self, node: NonNull<T>) {
		let other = IntrusiveList::singleton(node);
		self.merge(other);
	}

	pub fn root(&self) -> Option<&T> {
		self.root.as_ref().map(|root| unsafe { root.as_ref() })
	}

	pub fn extract_root(&mut self) -> Option<NonNull<T>> {
		let mut root = self.root?;
		let mut leftmost = *unsafe { root.as_mut() }.left_mut();
		let mut rightmost = *unsafe { root.as_mut() }.right_mut();
		if leftmost != root {
			*unsafe { leftmost.as_mut() }.right_mut() = rightmost;
			*unsafe { rightmost.as_mut() }.left_mut() = leftmost;
			initialize_elem(unsafe { root.as_mut() });
			self.root = Some(leftmost);
		} else {
			self.root = None;
		}
		Some(root)
	}

	pub fn drop_custom(&mut self, mut f: impl FnMut(NonNull<T>)) {
		let first_node = match self.root {
			Some(child) => child,
			None => return,
		};

		let mut node = first_node;
		loop {
			let next_node = *unsafe { node.as_mut() }.right_mut();
			f(node);
			if next_node == first_node {
				break;
			}
			node = next_node;
		}

		self.root = None;
	}

	pub fn merge(&mut self, other: Self) {
		let root = match (self.root, other.root) {
			(Some(mut self_rightmost), Some(mut other_leftmost)) => {
				let mut self_leftmost = *unsafe { self_rightmost.as_mut() }.right_mut();
				let mut other_rightmost = *unsafe { other_leftmost.as_mut() }.left_mut();
				*unsafe { self_rightmost.as_mut() }.right_mut() = other_leftmost;
				*unsafe { self_leftmost.as_mut() }.left_mut() = other_rightmost;
				*unsafe { other_leftmost.as_mut() }.left_mut() = self_rightmost;
				*unsafe { other_rightmost.as_mut() }.right_mut() = self_leftmost;
				Some(self_rightmost)
			},
			(Some(only), None) | (None, Some(only)) => Some(only),
			(None, None) => None,
		};
		self.root = root;
	}

	pub fn set_root(&mut self, root: NonNull<T>) {
		debug_assert!(self.root.is_some());
		self.root = Some(root);
	}
}

pub fn initialize_elem<T: IntrusiveListElem>(elem: &mut T) {
	*elem.left_mut() = NonNull::from(elem as &mut _);
	*elem.right_mut() = NonNull::from(elem as &mut _);
}
