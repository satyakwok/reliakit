//! Bounded collections that cannot hold an invalid number of elements.
//!
//! `BoundedVec` enforces a length range at every mutation; `RingBuffer` keeps
//! only the most recent N items. Run with:
//!
//! ```sh
//! cargo run -p reliakit-collections --example basic
//! ```

use reliakit_collections::{BoundedVec, RingBuffer};

fn main() {
    // A list that must hold between 1 and 3 elements.
    let mut queue: BoundedVec<&str, 1, 3> = BoundedVec::new(vec!["first"]).unwrap();
    queue.push("second").unwrap();
    queue.push("third").unwrap();
    println!("queue: {:?} (len {})", queue.as_slice(), queue.len());

    // A fourth element would break the upper bound, so it is refused.
    match queue.push("fourth") {
        Ok(()) => println!("pushed fourth"),
        Err(e) => println!("push refused: {e}"),
    }

    // Popping below the lower bound is refused too.
    queue.pop().unwrap();
    queue.pop().unwrap();
    match queue.pop() {
        Ok(item) => println!("popped {item}"),
        Err(e) => println!("pop refused: {e}"),
    }

    // A ring buffer of capacity 3 evicts the oldest item once it is full.
    let mut recent: RingBuffer<u32> = RingBuffer::new(3).unwrap();
    for n in 1..=5 {
        if let Some(evicted) = recent.push(n) {
            println!("ring evicted {evicted}");
        }
    }
    println!(
        "ring holds {} items; oldest = {:?}, newest = {:?}",
        recent.len(),
        recent.oldest(),
        recent.newest(),
    );
}
