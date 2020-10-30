use std::ptr::NonNull;

pub unsafe trait EmbeddedListElem {
	fn left_mut(&mut self) -> &mut NonNull<Self>;
	fn right_mut(&mut self) -> &mut NonNull<Self>;

	unsafe fn embedlist_initalize(&mut self)
	where Self: Sized {
		*self.left_mut() = NonNull::new(self as *mut _).unwrap();
		*self.right_mut() = NonNull::new(self as *mut _).unwrap();
	}
}

pub struct EmbedList<T> {
	child: Option<NonNull<T>>,
}

impl<T: EmbeddedListElem> EmbedList<T> {
	pub fn new() -> Self {
		EmbedList { child: None }
	}

	pub fn insert(&mut self, node: NonNull<T>, as_root: bool) {
		self.insert_anywhere(node);
		if as_root {
			self.child = Some(node);
		}
	}

	fn insert_anywhere(&mut self, mut value: NonNull<T>) {
		let mut leftmost = match self.child {
			Some(root) => root,
			None => {
				self.child = Some(value);
				return;
			},
		};
		let mut rightmost = *unsafe { leftmost.as_mut() }.right_mut();
		*unsafe { value.as_mut() }.left_mut() = rightmost;
		*unsafe { value.as_mut() }.right_mut() = leftmost;
		*unsafe { leftmost.as_mut() }.left_mut() = value;
		*unsafe { rightmost.as_mut() }.right_mut() = value;
	}

	pub fn root(&self) -> Option<&T> {
		self.child.as_ref().map(|root| unsafe { root.as_ref() })
	}

	pub fn extract_root(&mut self) -> Option<NonNull<T>> {
		let mut root = self.child?;
		let mut leftmost = *unsafe { root.as_mut() }.left_mut();
		let mut rightmost = *unsafe { root.as_mut() }.right_mut();
		if leftmost != root {
			*unsafe { leftmost.as_mut() }.right_mut() = rightmost;
			*unsafe { rightmost.as_mut() }.left_mut() = leftmost;
			unsafe { root.as_mut().embedlist_initalize() };
			self.child = Some(leftmost);
		} else {
			self.child = None;
		}
		Some(root)
	}

	pub fn drop_custom(&mut self, mut f: impl FnMut(NonNull<T>)) {
		let first_node = match self.child {
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

		self.child = None;
	}
}
