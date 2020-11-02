use fibheap::FibHeap;
use quickcheck_macros::quickcheck;
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
struct Reverse<T>(T);

impl<T: PartialOrd> PartialOrd for Reverse<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		other.0.partial_cmp(&self.0)
	}
}
impl<T: Ord+PartialOrd> Ord for Reverse<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		other.0.cmp(&self.0)
	}
}

#[quickcheck]
fn as_many_popped_as_inserted(xs: Vec<i32>) {
	let mut heap = FibHeap::new();
	for x in &xs {
		heap.insert(*x);
	}
	let mut count = 0;
	while heap.pop_min().is_some() {
		count += 1;
	}
	assert_eq!(count, xs.len());
}

#[quickcheck]
fn good_order(xs: Vec<i32>) {
	let mut h1 = FibHeap::new();
	let mut h2 = std::collections::BinaryHeap::new();
	for x in xs {
		h1.insert(x);
		h2.push(Reverse(x));
	}
	while let Some(x) = h2.pop() {
		assert_eq!(h1.pop_min(), Some(x.0));
	}
	assert_eq!(h1.pop_min(), None);
}
