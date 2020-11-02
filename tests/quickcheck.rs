use fibheap::FibHeap;
use quickcheck_macros::quickcheck;

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
