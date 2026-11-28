//! A counting global allocator, for the half of `LIM-2`, `LIM-3` and `LIM-7`
//! that is a **measurement**.
//!
//! *Checked before allocating* is a claim about what the process held, not about
//! which error came back, and no assertion over a return value reaches it. A
//! loader that inflates a quarter of a gigabyte and then reports a limit has
//! obeyed the letter of `LIM-3` and has the decompression bomb the criterion
//! exists to refuse.
//!
//! # It lives in the test binaries and nowhere else
//!
//! `rigel-format` does not know this exists and must not: a `#[global_allocator]`
//! is a property of a binary, and a library that depended on one would impose it
//! on both applications. This file is included by `tests/acceptance/main.rs` and
//! by `tests/fuzz.rs`, each of which registers it for its own binary.
//!
//! # Why the counters are per thread
//!
//! **`cargo test` runs the tests in one binary concurrently, on a thread each.**
//! A process-global peak counter would therefore report whatever the whole
//! binary happened to be doing while the measured call ran — in a suite that
//! loads fixtures in parallel, mostly *other tests*. That is worse than no
//! counter: it is a figure that looks like a measurement, passes while the
//! suite is small, and fails later for a reason nobody can attribute.
//!
//! The alternative was a global counter with the measuring tests serialised
//! behind a mutex. It was rejected: it needs every future measuring test to
//! remember to take the lock — the failure is silent when one forgets — and it
//! slows the suite in exchange for nothing a thread-local does not already give.
//!
//! **What a per-thread counter cannot see** is a thread the measured code
//! spawned. `rigel-format` spawns none; if it ever does, this file is where that
//! assumption is written down and the measurement has to change with it.
//!
//! # What is counted
//!
//! Bytes requested through Rust's global allocator, on this thread, as a net
//! figure above where the measured region started. A `realloc` is counted as the
//! new block arriving **before** the old one leaves, because that is what a
//! growing `Vec` actually holds while it copies — the transient doubling is real
//! and is exactly the spike a limit enforced by `read_to_end` would hide.

#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    /// Bytes this thread holds, relative to the start of the measured region.
    /// Signed, because a region may free something allocated before it began.
    static LIVE: Cell<i64> = const { Cell::new(0) };
    /// The high-water mark of [`LIVE`] since the region began.
    static PEAK: Cell<i64> = const { Cell::new(0) };
}

/// The system allocator, with a per-thread byte counter around it.
pub struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            took(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            took(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        gave_back(layout.size());
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = unsafe { System.realloc(ptr, layout, new_size) };
        if moved.is_null() {
            return moved;
        }
        // The order is the point: both blocks exist while the contents are
        // copied, so the peak carries the sum rather than the difference.
        took(new_size);
        gave_back(layout.size());
        moved
    }
}

/// `try_with` rather than `with`: the counters outlive nothing and have no
/// destructor, so this cannot fail today — but an allocation during thread
/// teardown must never be the thing that panics inside the allocator.
fn took(bytes: usize) {
    let _ = LIVE.try_with(|live| {
        let now = live.get().saturating_add(bytes as i64);
        live.set(now);
        let _ = PEAK.try_with(|peak| {
            if now > peak.get() {
                peak.set(now);
            }
        });
    });
}

fn gave_back(bytes: usize) {
    let _ = LIVE.try_with(|live| live.set(live.get().saturating_sub(bytes as i64)));
}

/// Runs `f` and reports the peak bytes this thread held above where it started.
///
/// Nests: an inner region restores the outer one's counters and folds its own
/// peak into them, so a measurement inside a measurement reports both honestly.
pub fn peak_bytes_during<T>(f: impl FnOnce() -> T) -> (T, u64) {
    let outer_live = LIVE.with(Cell::get);
    let outer_peak = PEAK.with(Cell::get);
    LIVE.with(|live| live.set(0));
    PEAK.with(|peak| peak.set(0));

    let value = f();

    let peak = PEAK.with(Cell::get);
    let live = LIVE.with(Cell::get);
    LIVE.with(|c| c.set(outer_live.saturating_add(live)));
    PEAK.with(|c| c.set(outer_peak.max(outer_live.saturating_add(peak))));

    (value, peak.max(0) as u64)
}

/// The peak bytes `f` held, discarding its value.
pub fn peak_bytes(f: impl FnOnce()) -> u64 {
    peak_bytes_during(f).1
}

/// The allocator's own tests, which run in **every binary that includes this
/// file** — which is the point. A `#[global_allocator]` that a binary forgot to
/// register reports a peak of zero, and a peak of zero passes every bound in
/// `limits.rs` vacuously. These are what stop that.
#[cfg(test)]
mod tests {
    use super::*;

    /// A block this size is far above the noise of anything else the thread
    /// does, so the counter either sees it or is not wired up.
    const BIG: usize = 32 * 1024 * 1024;

    #[test]
    fn an_allocation_inside_the_region_is_counted() {
        let peak = peak_bytes(|| {
            let block = vec![0u8; BIG];
            std::hint::black_box(&block);
        });
        assert!(
            peak >= BIG as u64,
            "a {BIG}-byte allocation reported a peak of {peak}: the allocator is \
             not registered in this binary, and every bound asserted against it \
             passes vacuously"
        );
    }

    #[test]
    fn a_region_that_holds_nothing_reports_almost_nothing() {
        let peak = peak_bytes(|| {
            let block = vec![0u8; BIG];
            std::hint::black_box(&block);
            drop(block);
        });
        assert!(
            peak >= BIG as u64,
            "the peak is a high-water mark, not what survived"
        );
        let after = peak_bytes(|| {
            std::hint::black_box(1u64);
        });
        assert!(after < 4096, "an empty region reported {after}");
    }

    /// A `Vec` that doubles its way up holds the old block and the new one while
    /// it copies, so its peak exceeds its final size. This is the transient that
    /// put `archive::read`'s old shape at three times the payload limit, and the
    /// allocator has to see it or the measurement understates the thing it was
    /// built to catch.
    #[test]
    fn a_growing_vec_peaks_above_what_it_ends_up_holding() {
        let peak = peak_bytes(|| {
            let mut grown: Vec<u8> = Vec::new();
            for _ in 0..BIG / 4096 {
                grown.extend_from_slice(&[0u8; 4096]);
            }
            std::hint::black_box(&grown);
        });
        assert!(
            peak > BIG as u64,
            "a vec grown to {BIG} bytes peaked at only {peak}: reallocation is not \
             being counted"
        );
    }

    /// **The test that makes the per-thread decision self-enforcing.** Another
    /// thread allocates a large block for exactly as long as the measured region
    /// runs. A process-global counter reports it; a per-thread one does not, and
    /// the whole reason this file is thread-local is that `cargo test` runs the
    /// tests beside each other.
    #[test]
    fn another_thread_allocating_is_not_this_thread_s_peak() {
        use std::sync::{Arc, Barrier};

        let entered = Arc::new(Barrier::new(2));
        let holding = Arc::new(Barrier::new(2));
        let (their_entered, their_holding) = (Arc::clone(&entered), Arc::clone(&holding));
        let other = std::thread::spawn(move || {
            their_entered.wait();
            let block = vec![0u8; BIG];
            std::hint::black_box(&block);
            their_holding.wait();
        });

        let peak = peak_bytes(|| {
            entered.wait();
            // The other thread allocates here, and is still holding it when it
            // reaches the second barrier.
            holding.wait();
        });
        other.join().expect("the other thread finishes");

        assert!(
            peak < 1024 * 1024,
            "a {BIG}-byte allocation on another thread showed up as {peak} bytes \
             here: the counters are not per thread, and every measurement in this \
             suite is reporting whatever the tests beside it were doing"
        );
    }

    #[test]
    fn a_measurement_inside_a_measurement_reports_both() {
        let mut inner = 0;
        let outer = peak_bytes(|| {
            let held = vec![0u8; BIG];
            inner = peak_bytes(|| {
                let nested = vec![0u8; BIG];
                std::hint::black_box(&nested);
            });
            std::hint::black_box(&held);
        });
        assert!(inner >= BIG as u64, "the inner region saw its own block");
        assert!(
            outer >= 2 * BIG as u64,
            "the outer region saw both blocks, and reported {outer}"
        );
    }
}
