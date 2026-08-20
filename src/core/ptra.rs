//! Generic pointer array with holes
//!
//! [`Ptra`] is a dynamically resizable array whose slots may be empty. Unlike
//! a `Vec`, removing an element leaves a hole by default; compaction is an
//! explicit operation. This lets callers rearrange large collections without
//! paying for a shift on every removal.
//!
//! C Leptonica equivalent: `L_PTRA` in `ptra.c`.

/// How [`Ptra::insert`] makes room when the target slot is occupied.
///
/// C equivalent: `L_AUTO_DOWNSHIFT` / `L_MIN_DOWNSHIFT` / `L_FULL_DOWNSHIFT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownShift {
    /// Pick between the two below based on how many holes there are.
    Auto,
    /// Shift down only as far as the nearest hole.
    Min,
    /// Shift every element from the insertion point to the end.
    Full,
}

/// Whether [`Ptra::remove`] closes the gap it leaves behind.
///
/// C equivalent: `L_NO_COMPACTION` / `L_COMPACTION`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compaction {
    /// Leave a hole at the removed index.
    No,
    /// Move everything after the index down by one.
    Yes,
}

/// C `DefaultInitPtraSize`.
#[allow(dead_code)]
const DEFAULT_INIT_SIZE: usize = 20;

/// A dynamically resizable array that may contain holes.
///
/// C Leptonica equivalent: `L_PTRA`.
#[derive(Debug, Clone)]
pub struct Ptra<T> {
    /// The backing slots; `None` is a hole. Length is C's `nalloc`.
    array: Vec<Option<T>>,
    /// Largest index holding an item, or -1 when empty. C's `imax`.
    imax: i32,
    /// Number of occupied slots. C's `nactual`.
    nactual: i32,
}

impl<T> Default for Ptra<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Ptra<T> {
    /// Create an empty array with the default capacity.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create an empty array with room for `n` items.
    ///
    /// C rounds a non-positive `n` up to 20, and so does this.
    ///
    /// C Leptonica equivalent: `ptraCreate`
    pub fn with_capacity(n: usize) -> Self {
        let _ = n;
        unimplemented!("ptraCreate")
    }

    /// Largest index holding an item, or -1 when the array is empty.
    ///
    /// C Leptonica equivalent: `ptraGetMaxIndex`
    pub fn max_index(&self) -> i32 {
        self.imax
    }

    /// Number of occupied slots, which excludes holes.
    ///
    /// C Leptonica equivalent: `ptraGetActualCount`
    pub fn actual_count(&self) -> i32 {
        self.nactual
    }

    /// Whether the array holds no items.
    pub fn is_empty(&self) -> bool {
        self.nactual == 0
    }

    /// Number of slots, including holes. C's `nalloc`.
    pub fn capacity(&self) -> usize {
        self.array.len()
    }

    /// Borrow the item at `index`, or `None` for a hole or out-of-range index.
    ///
    /// C Leptonica equivalent: `ptraGetPtrToItem`
    pub fn get(&self, index: i32) -> Option<&T> {
        if index < 0 {
            return None;
        }
        self.array.get(index as usize).and_then(|s| s.as_ref())
    }

    /// C `ptraExtendArray`: double the number of slots.
    #[allow(dead_code)]
    fn extend_array(&mut self) {
        let new_len = self.array.len() * 2;
        self.array.resize_with(new_len, || None);
    }

    /// Append `item` just past the current maximum index.
    ///
    /// C Leptonica equivalent: `ptraAdd`
    pub fn add(&mut self, item: T) {
        let _ = item;
        unimplemented!("ptraAdd")
    }

    /// Insert `item` at `index`, shifting occupied slots down as needed.
    ///
    /// Inserting into a hole or past the maximum index never shifts anything.
    ///
    /// C Leptonica equivalent: `ptraInsert`
    pub fn insert(&mut self, index: i32, item: T, shift: DownShift) {
        let _ = (index, item, shift);
        unimplemented!("ptraInsert")
    }

    /// C's `ptraInsert` accepts a NULL item, which only bumps bookkeeping.
    /// [`Ptra::swap`] relies on that, so keep the general form private.
    #[allow(dead_code)]
    fn insert_opt(&mut self, index: i32, item: Option<T>, shift: DownShift) {
        if item.is_some() {
            self.nactual += 1;
        }
        if index as usize == self.array.len() {
            self.extend_array();
        }

        let index = index as usize;
        let imax = self.imax;

        // Inserting into a hole or beyond the last item: no shift needed.
        if self.array[index].is_none() {
            let was_some = item.is_some();
            self.array[index] = item;
            if was_some && index as i32 > imax {
                self.imax = index as i32;
            }
            return;
        }

        if imax >= self.array.len() as i32 - 1 {
            self.extend_array();
        }

        // With no holes there is nothing to shift into, so go all the way.
        let shift = if imax + 1 == self.nactual {
            DownShift::Full
        } else if shift == DownShift::Auto {
            if imax < 10 {
                DownShift::Full
            } else {
                // C computes `(imax - index) / imax` with integer division,
                // so this is 1 only when index is 0 and 0 otherwise.
                let nexpected =
                    (imax - self.nactual) as f32 * ((imax - index as i32) / imax) as f32;
                if nexpected > 2.0 {
                    DownShift::Min
                } else {
                    DownShift::Full
                }
            }
        } else {
            shift
        };

        let ihole = if shift == DownShift::Min {
            let mut ihole = index as i32 + 1;
            while ihole <= imax {
                if self.array[ihole as usize].is_none() {
                    break;
                }
                ihole += 1;
            }
            ihole
        } else {
            imax + 1
        };

        let mut i = ihole;
        while i > index as i32 {
            self.array[i as usize] = self.array[(i - 1) as usize].take();
            i -= 1;
        }
        self.array[index] = item;
        if ihole == imax + 1 {
            self.imax += 1;
        }
    }

    /// Take the item at `index`, leaving a hole or closing the gap.
    ///
    /// Returns `None` when the slot was already a hole. Removing the last item
    /// always lowers the maximum index to the next occupied slot, regardless of
    /// `flag`.
    ///
    /// C Leptonica equivalent: `ptraRemove`
    pub fn remove(&mut self, index: i32, flag: Compaction) -> Option<T> {
        let _ = (index, flag);
        unimplemented!("ptraRemove")
    }

    /// Take the item at the maximum index.
    ///
    /// C Leptonica equivalent: `ptraRemoveLast`
    pub fn remove_last(&mut self) -> Option<T> {
        unimplemented!("ptraRemoveLast")
    }

    /// Put `item` at `index` and return whatever was there.
    ///
    /// C Leptonica equivalent: `ptraReplace` with `freeflag = FALSE`
    pub fn replace(&mut self, index: i32, item: Option<T>) -> Option<T> {
        let _ = (index, item);
        unimplemented!("ptraReplace")
    }

    /// Exchange the items at two indices.
    ///
    /// C Leptonica equivalent: `ptraSwap`
    pub fn swap(&mut self, index1: i32, index2: i32) {
        let _ = (index1, index2);
        unimplemented!("ptraSwap")
    }

    /// Close every hole, preserving order.
    ///
    /// C Leptonica equivalent: `ptraCompactArray`
    pub fn compact(&mut self) {
        unimplemented!("ptraCompactArray")
    }

    /// Reverse the order of the items.
    ///
    /// C Leptonica equivalent: `ptraReverse`
    pub fn reverse(&mut self) {
        unimplemented!("ptraReverse")
    }

    /// Move every item of `other` onto the end of this array.
    ///
    /// C Leptonica equivalent: `ptraJoin`
    pub fn join(&mut self, other: &mut Ptra<T>) {
        let _ = other;
        unimplemented!("ptraJoin")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(n: usize) -> Ptra<i32> {
        let mut pa = Ptra::with_capacity(n);
        for i in 0..n {
            pa.add(i as i32);
        }
        pa
    }

    /// C `ptraCreate`: a non-positive request becomes the default size of 20,
    /// and a fresh array reports imax = -1.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_create_and_add() {
        let pa: Ptra<i32> = Ptra::new();
        assert_eq!(pa.capacity(), 20);
        assert_eq!(pa.max_index(), -1);
        assert_eq!(pa.actual_count(), 0);

        let pa = filled(5);
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.actual_count(), 5);
        assert_eq!(pa.get(2), Some(&2));
    }

    /// C `ptraAdd` doubles the backing array when it runs out of slots.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_add_extends() {
        let pa = filled(25);
        assert_eq!(pa.capacity(), 40);
        assert_eq!(pa.max_index(), 24);
        assert_eq!(pa.actual_count(), 25);
    }

    /// C `ptraRemove` with `L_NO_COMPACTION` leaves a hole; imax only drops
    /// when the last item is taken, and then down to the next occupied slot.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_no_compaction() {
        let mut pa = filled(5);
        assert_eq!(pa.remove(1, Compaction::No), Some(1));
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.actual_count(), 4);
        assert_eq!(pa.get(1), None);
        assert_eq!(pa.get(2), Some(&2));

        // Removing an existing hole yields nothing and changes no counts.
        assert_eq!(pa.remove(1, Compaction::No), None);
        assert_eq!(pa.actual_count(), 4);

        // Taking the last item walks imax back over the holes below it.
        pa.remove(3, Compaction::No);
        assert_eq!(pa.remove(4, Compaction::No), Some(4));
        assert_eq!(pa.max_index(), 2);
    }

    /// C `ptraRemove` with `L_COMPACTION` closes the gap up to imax.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_with_compaction() {
        let mut pa = filled(5);
        assert_eq!(pa.remove(1, Compaction::Yes), Some(1));
        assert_eq!(pa.max_index(), 3);
        assert_eq!(pa.actual_count(), 4);
        assert_eq!(pa.get(1), Some(&2));
        assert_eq!(pa.get(3), Some(&4));
    }

    /// C `ptraInsert` into a hole never shifts anything.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_insert_into_hole() {
        let mut pa = filled(5);
        pa.remove(2, Compaction::No);
        pa.insert(2, 99, DownShift::Min);
        assert_eq!(pa.get(2), Some(&99));
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.actual_count(), 5);
    }

    /// C `ptraInsert` at an occupied slot: with no holes it always does a
    /// full downshift, so imax grows by one.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_insert_full_downshift() {
        let mut pa = filled(5);
        pa.insert(0, 99, DownShift::Min);
        assert_eq!(pa.max_index(), 5);
        assert_eq!(pa.actual_count(), 6);
        assert_eq!(pa.get(0), Some(&99));
        assert_eq!(pa.get(1), Some(&0));
        assert_eq!(pa.get(5), Some(&4));
    }

    /// With a hole available, `DownShift::Min` shifts only that far and imax
    /// stays put.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_insert_min_downshift_stops_at_hole() {
        let mut pa = filled(5);
        pa.remove(3, Compaction::No);
        pa.insert(1, 99, DownShift::Min);
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.get(1), Some(&99));
        assert_eq!(pa.get(2), Some(&1));
        assert_eq!(pa.get(3), Some(&2));
        assert_eq!(pa.get(4), Some(&4));
    }

    /// C `ptraCompactArray` removes every hole and is a no-op when there are
    /// none.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_compact() {
        let mut pa = filled(5);
        pa.remove(1, Compaction::No);
        pa.remove(3, Compaction::No);
        pa.compact();
        assert_eq!(pa.max_index(), 2);
        assert_eq!(pa.actual_count(), 3);
        assert_eq!(
            (pa.get(0), pa.get(1), pa.get(2)),
            (Some(&0), Some(&2), Some(&4))
        );

        pa.compact();
        assert_eq!(pa.max_index(), 2);
    }

    /// C `ptraSwap` exchanges two items and leaves the counts unchanged.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_swap_and_reverse() {
        let mut pa = filled(5);
        pa.swap(0, 4);
        assert_eq!(pa.get(0), Some(&4));
        assert_eq!(pa.get(4), Some(&0));
        assert_eq!(pa.actual_count(), 5);
        assert_eq!(pa.max_index(), 4);

        let mut pa = filled(5);
        pa.reverse();
        let got: Vec<i32> = (0..=pa.max_index())
            .filter_map(|i| pa.get(i).copied())
            .collect();
        assert_eq!(got, vec![4, 3, 2, 1, 0]);
    }

    /// C `ptraRemoveLast` takes the item at imax.
    #[test]
    #[ignore = "not yet implemented"]
    fn test_remove_last_and_join() {
        let mut pa = filled(3);
        assert_eq!(pa.remove_last(), Some(2));
        assert_eq!(pa.max_index(), 1);

        let mut other = filled(2);
        pa.join(&mut other);
        assert_eq!(pa.actual_count(), 4);
        assert_eq!(other.actual_count(), 0);
        let got: Vec<i32> = (0..=pa.max_index())
            .filter_map(|i| pa.get(i).copied())
            .collect();
        assert_eq!(got, vec![0, 1, 0, 1]);
    }
}
