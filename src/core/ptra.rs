//! Generic pointer array with holes
//!
//! [`Ptra`] is a dynamically resizable array whose slots may be empty. Unlike
//! a `Vec`, removing an element leaves a hole by default; compaction is an
//! explicit operation. This lets callers rearrange large collections without
//! paying for a shift on every removal.
//!
//! C Leptonica equivalent: `L_PTRA` in `ptra.c`.

use crate::core::error::{Error, Result};

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
    /// `n == 0` falls back to 20 slots, matching what C does for a
    /// non-positive request.
    ///
    /// C Leptonica equivalent: `ptraCreate`
    pub fn with_capacity(n: usize) -> Self {
        let n = if n == 0 { DEFAULT_INIT_SIZE } else { n };
        let mut array = Vec::with_capacity(n);
        array.resize_with(n, || None);
        Self {
            array,
            imax: -1,
            nactual: 0,
        }
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
    fn extend_array(&mut self) {
        let new_len = self.array.len() * 2;
        self.array.resize_with(new_len, || None);
    }

    /// Append `item` just past the current maximum index.
    ///
    /// C Leptonica equivalent: `ptraAdd`
    pub fn add(&mut self, item: T) {
        if self.imax >= self.array.len() as i32 - 1 {
            self.extend_array();
        }
        self.array[(self.imax + 1) as usize] = Some(item);
        self.imax += 1;
        self.nactual += 1;
    }

    /// Insert `item` at `index`, shifting occupied slots down as needed.
    ///
    /// Inserting into a hole or past the maximum index never shifts anything.
    ///
    /// `index` may be at most the current slot count (C's `nalloc`), which is
    /// the same bound C enforces; inserting exactly at that boundary grows the
    /// array. This is deliberately tighter than [`Ptra::add`], which always
    /// appends and grows on demand.
    ///
    /// C Leptonica equivalent: `ptraInsert`
    pub fn insert(&mut self, index: i32, item: T, shift: DownShift) -> Result<()> {
        if index < 0 || index as usize > self.array.len() {
            return Err(Error::IndexOutOfBounds {
                index: index.max(0) as usize,
                len: self.array.len(),
            });
        }

        self.nactual += 1;
        if index as usize == self.array.len() {
            self.extend_array();
        }

        let index = index as usize;
        let imax = self.imax;

        // Inserting into a hole or beyond the last item: no shift needed.
        if self.array[index].is_none() {
            self.array[index] = Some(item);
            if index as i32 > imax {
                self.imax = index as i32;
            }
            return Ok(());
        }

        if imax >= self.array.len() as i32 - 1 {
            self.extend_array();
        }

        // C decides "there are no holes" with `imax + 1 == nactual`, and
        // nactual already counts the item being inserted. So a single hole is
        // not enough to keep a min downshift.
        let shift = if imax + 1 == self.nactual {
            DownShift::Full
        } else if shift == DownShift::Auto {
            if imax < 10 {
                DownShift::Full
            } else {
                // Two deliberate oddities carried over from C: the hole
                // count is `imax - nactual` (one less than the actual number
                // of holes below imax), and `(imax - index) / imax` is
                // integer division, so it is 1 only when index is 0.
                // Together these make Auto pick Min only in narrow cases.
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
            // Run down looking for the first hole to shift into.
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
        self.array[index] = Some(item);
        if ihole == imax + 1 {
            self.imax += 1;
        }
        Ok(())
    }

    /// Take the item at `index`, leaving a hole or closing the gap.
    ///
    /// Returns `None` when the slot was already a hole. Removing the last item
    /// always lowers the maximum index to the next occupied slot, regardless of
    /// `flag`.
    ///
    /// C Leptonica equivalent: `ptraRemove`
    pub fn remove(&mut self, index: i32, flag: Compaction) -> Result<Option<T>> {
        let imax = self.imax;
        if index < 0 || index > imax {
            return Err(Error::IndexOutOfBounds {
                index: index.max(0) as usize,
                len: (imax + 1).max(0) as usize,
            });
        }

        let item = self.array[index as usize].take();
        if item.is_some() {
            self.nactual -= 1;
        }

        let fromend = index == imax;
        if fromend {
            let mut i = index - 1;
            while i >= 0 {
                if self.array[i as usize].is_some() {
                    break;
                }
                i -= 1;
            }
            self.imax = i;
        } else if flag == Compaction::Yes {
            let mut icurrent = index;
            for i in (index + 1)..=imax {
                if self.array[i as usize].is_some() {
                    self.array[icurrent as usize] = self.array[i as usize].take();
                    icurrent += 1;
                }
            }
            self.imax = icurrent - 1;
        }

        Ok(item)
    }

    /// Take the item at the maximum index.
    ///
    /// C Leptonica equivalent: `ptraRemoveLast`
    pub fn remove_last(&mut self) -> Option<T> {
        if self.imax < 0 {
            return None;
        }
        self.remove(self.imax, Compaction::No)
            .expect("imax is in range by construction")
    }

    /// Put `item` at `index` and return whatever was there.
    ///
    /// C Leptonica equivalent: `ptraReplace` with `freeflag = FALSE`
    pub fn replace(&mut self, index: i32, item: Option<T>) -> Result<Option<T>> {
        if index < 0 || index > self.imax {
            return Err(Error::IndexOutOfBounds {
                index: index.max(0) as usize,
                len: (self.imax + 1).max(0) as usize,
            });
        }
        let olditem = self.array[index as usize].take();
        let had_item = item.is_some();
        self.array[index as usize] = item;
        if !had_item && olditem.is_some() {
            self.nactual -= 1;
        } else if had_item && olditem.is_none() {
            self.nactual += 1;
        }
        Ok(olditem)
    }

    /// Exchange the contents of two slots, either of which may be a hole.
    ///
    /// C Leptonica equivalent: `ptraSwap`
    ///
    /// C routes this through `ptraRemove` → `ptraReplace` → `ptraInsert`,
    /// which for two occupied slots is just an exchange. That route breaks
    /// when the first index is the last occupied slot and the second sits
    /// below a run of holes: the remove drops `imax` past those holes, the
    /// replace then rejects the now out-of-range index, and the item is
    /// dropped on the floor. Exchanging the slots directly gives the same
    /// result in every case C handles, without losing an item.
    pub fn swap(&mut self, index1: i32, index2: i32) -> Result<()> {
        if index1 == index2 {
            return Ok(());
        }
        let imax = self.imax;
        for index in [index1, index2] {
            if index < 0 || index > imax {
                return Err(Error::IndexOutOfBounds {
                    index: index.max(0) as usize,
                    len: (imax + 1).max(0) as usize,
                });
            }
        }
        self.array.swap(index1 as usize, index2 as usize);

        // An exchange never changes the item count, but it can empty the
        // tail, in which case imax falls back to the last occupied slot.
        if self.array[imax as usize].is_none() {
            let mut i = imax - 1;
            while i >= 0 && self.array[i as usize].is_none() {
                i -= 1;
            }
            self.imax = i;
        }
        Ok(())
    }

    /// Close every hole, preserving order.
    ///
    /// C Leptonica equivalent: `ptraCompactArray`
    pub fn compact(&mut self) {
        let imax = self.imax;
        if imax + 1 == self.nactual {
            return;
        }
        let mut index = 0i32;
        for i in 0..=imax {
            if self.array[i as usize].is_some() {
                self.array[index as usize] = self.array[i as usize].take();
                index += 1;
            }
        }
        self.imax = index - 1;
    }

    /// Reverse the order of the items.
    ///
    /// C Leptonica equivalent: `ptraReverse`
    pub fn reverse(&mut self) {
        let imax = self.imax;
        for i in 0..((imax + 1) / 2) {
            self.swap(i, imax - i)
                .expect("both indices are within [0, imax] by construction");
        }
    }

    /// Move every item of `other` onto the end of this array.
    ///
    /// C Leptonica equivalent: `ptraJoin`
    pub fn join(&mut self, other: &mut Ptra<T>) {
        let imax = other.imax;
        for i in 0..=imax {
            let item = other
                .remove(i, Compaction::No)
                .expect("i is within [0, imax] by construction");
            if let Some(item) = item {
                self.add(item);
            }
        }
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

    /// C `ptraAdd` doubles the backing array only once it would overflow:
    /// the check is `imax >= nalloc - 1` *before* the store, so filling an
    /// array to exactly its capacity does not extend it.
    #[test]
    fn test_add_extends() {
        let pa = filled(25);
        assert_eq!(pa.capacity(), 25);
        assert_eq!(pa.max_index(), 24);
        assert_eq!(pa.actual_count(), 25);

        // Adding one more than the capacity doubles it.
        let mut pa = filled(5);
        assert_eq!(pa.capacity(), 5);
        pa.add(5);
        assert_eq!(pa.capacity(), 10);
        assert_eq!(pa.max_index(), 5);
    }

    /// C `ptraRemove` with `L_NO_COMPACTION` leaves a hole; imax only drops
    /// when the last item is taken, and then down to the next occupied slot.
    #[test]
    fn test_remove_no_compaction() {
        let mut pa = filled(5);
        assert_eq!(pa.remove(1, Compaction::No).expect("remove"), Some(1));
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.actual_count(), 4);
        assert_eq!(pa.get(1), None);
        assert_eq!(pa.get(2), Some(&2));

        // Removing an existing hole yields nothing and changes no counts.
        assert_eq!(pa.remove(1, Compaction::No).expect("remove"), None);
        assert_eq!(pa.actual_count(), 4);

        // Taking the last item walks imax back over the holes below it.
        pa.remove(3, Compaction::No).expect("remove");
        assert_eq!(pa.remove(4, Compaction::No).expect("remove"), Some(4));
        assert_eq!(pa.max_index(), 2);
    }

    /// C `ptraRemove` with `L_COMPACTION` closes the gap up to imax.
    #[test]
    fn test_remove_with_compaction() {
        let mut pa = filled(5);
        assert_eq!(pa.remove(1, Compaction::Yes).expect("remove"), Some(1));
        assert_eq!(pa.max_index(), 3);
        assert_eq!(pa.actual_count(), 4);
        assert_eq!(pa.get(1), Some(&2));
        assert_eq!(pa.get(3), Some(&4));
    }

    /// C `ptraInsert` into a hole never shifts anything.
    #[test]
    fn test_insert_into_hole() {
        let mut pa = filled(5);
        pa.remove(2, Compaction::No).expect("remove");
        pa.insert(2, 99, DownShift::Min).expect("insert");
        assert_eq!(pa.get(2), Some(&99));
        assert_eq!(pa.max_index(), 4);
        assert_eq!(pa.actual_count(), 5);
    }

    /// C `ptraInsert` at an occupied slot: with no holes it always does a
    /// full downshift, so imax grows by one.
    #[test]
    fn test_insert_full_downshift() {
        let mut pa = filled(5);
        pa.insert(0, 99, DownShift::Min).expect("insert");
        assert_eq!(pa.max_index(), 5);
        assert_eq!(pa.actual_count(), 6);
        assert_eq!(pa.get(0), Some(&99));
        assert_eq!(pa.get(1), Some(&0));
        assert_eq!(pa.get(5), Some(&4));
    }

    /// With a hole still available after the insert is accounted for,
    /// `DownShift::Min` shifts only that far and imax stays put.
    #[test]
    fn test_insert_min_downshift_stops_at_hole() {
        let mut pa = filled(6);
        pa.remove(2, Compaction::No).expect("remove");
        pa.remove(4, Compaction::No).expect("remove");
        // [0, 1, _, 3, _, 5], imax = 5, nactual = 4
        pa.insert(0, 99, DownShift::Min).expect("insert");
        assert_eq!(pa.max_index(), 5);
        assert_eq!(pa.actual_count(), 5);
        assert_eq!(pa.get(0), Some(&99));
        assert_eq!(pa.get(1), Some(&0));
        assert_eq!(pa.get(2), Some(&1));
        assert_eq!(pa.get(3), Some(&3));
        assert_eq!(pa.get(4), None);
        assert_eq!(pa.get(5), Some(&5));
    }

    /// C decides "there are no holes" with `imax + 1 == nactual`, and nactual
    /// has already been incremented for the item being inserted. So a single
    /// hole is not enough: `DownShift::Min` is overridden to a full downshift
    /// and imax grows.
    #[test]
    fn test_insert_min_downgrades_to_full_with_one_hole() {
        let mut pa = filled(5);
        pa.remove(3, Compaction::No).expect("remove");
        // [0, 1, 2, _, 4], imax = 4, nactual = 4
        pa.insert(1, 99, DownShift::Min).expect("insert");
        assert_eq!(pa.max_index(), 5);
        assert_eq!(pa.get(1), Some(&99));
        assert_eq!(pa.get(2), Some(&1));
        assert_eq!(pa.get(3), Some(&2));
        assert_eq!(pa.get(4), None);
        assert_eq!(pa.get(5), Some(&4));
    }

    /// C `ptraCompactArray` removes every hole and is a no-op when there are
    /// none.
    #[test]
    fn test_compact() {
        let mut pa = filled(5);
        pa.remove(1, Compaction::No).expect("remove");
        pa.remove(3, Compaction::No).expect("remove");
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
    fn test_swap_and_reverse() {
        let mut pa = filled(5);
        pa.swap(0, 4).expect("swap");
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

    /// Swapping the last item into a hole below it must leave the array
    /// consistent. C loses the item here: its `ptraRemove` drops imax past
    /// the holes, the following `ptraReplace` then rejects the (now
    /// out-of-range) index and returns NULL, and the item is never stored.
    #[test]
    fn test_swap_last_into_hole() {
        let mut pa = filled(4);
        pa.remove(1, Compaction::No).expect("remove");
        pa.remove(2, Compaction::No).expect("remove");
        // [0, _, _, 3], imax = 3, nactual = 2
        pa.swap(3, 1).expect("swap");
        assert_eq!(pa.get(0), Some(&0));
        assert_eq!(pa.get(1), Some(&3));
        assert_eq!(pa.get(2), None);
        assert_eq!(pa.get(3), None);
        assert_eq!(pa.actual_count(), 2);
        // The tail became a hole, so imax drops to the last occupied slot.
        assert_eq!(pa.max_index(), 1);
    }

    /// Swapping two holes is a no-op, and swapping a hole with an item just
    /// moves the item.
    #[test]
    fn test_swap_with_holes() {
        let mut pa = filled(3);
        pa.remove(1, Compaction::No).expect("remove");
        pa.swap(0, 1).expect("swap");
        assert_eq!(pa.get(0), None);
        assert_eq!(pa.get(1), Some(&0));
        assert_eq!(pa.actual_count(), 2);
        assert_eq!(pa.max_index(), 2);
    }

    /// Out-of-range indices are reported, not asserted, matching the other
    /// core collections. C returns an error code in the same situations.
    #[test]
    fn test_out_of_range_reports_error() {
        let mut pa = filled(3);
        assert!(pa.remove(3, Compaction::No).is_err());
        assert!(pa.remove(-1, Compaction::No).is_err());
        assert!(pa.replace(3, Some(9)).is_err());
        assert!(pa.swap(0, 3).is_err());
        assert!(pa.insert(-1, 9, DownShift::Min).is_err());

        // A rejected insert must not have counted the item.
        let before = pa.actual_count();
        assert!(pa.insert(-1, 9, DownShift::Min).is_err());
        assert_eq!(pa.actual_count(), before);
    }

    /// C `ptraRemoveLast` takes the item at imax.
    #[test]
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
