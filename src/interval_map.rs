use std::ops::Range;

use num::{Bounded, Num};

#[derive(Debug, Clone, PartialEq)]
pub struct IntIntervalMap<Idx, V> {
    intervals: Vec<(Idx, V)>,
}

impl<Idx, V> IntervalMap<Idx, V> for IntIntervalMap<Idx, V>
where
    Idx: Copy + PartialOrd + Num + Bounded,
    V: Copy + PartialEq,
{
    fn assign(&mut self, range: Range<Idx>, new_value: V) {
        let lower_thresholds: Vec<Idx> = [Idx::min_value()]
            .iter()
            .chain(self.intervals.iter().map(|(idx, _)| idx))
            .cloned()
            .collect();

        let drain = self.intervals.drain(..);

        let mut result = vec![];

        for (lower, (upper, value)) in lower_thresholds.iter().zip(drain) {
            let start_in = range.start >= *lower;
            let end_in = range.end < upper;
            match (start_in, end_in) {
                // range is contained in current interval
                (true, true) => {
                    result.push((range.start, value));
                    result.push((range.end, new_value));
                    result.push((upper, value));
                }
                // range is greater or overlaps to the next
                (true, false) => {
                    // range starts in current interval
                    if range.start < upper {
                        result.push((range.start, value));
                        result.push((upper, new_value));
                    // range is greater than current interval
                    } else {
                        result.push((upper, value));
                    }
                }
                // range completes earlier or overlaps to the current interval
                (false, true) => {
                    if range.end < *lower {
                        result.push((upper, value))
                    } else {
                        result.push((range.end, new_value));
                        result.push((upper, value));
                    }
                }
                // current interval is contained in range
                (false, false) => {
                    result.push((range.end, new_value));
                }
            }
        }
        self.intervals = result;
        self.defrag();
    }

    fn assign_single(&mut self, single: Idx, value: V) {
        self.assign(single..single + Idx::one(), value);
    }

    fn get(&self, index: Idx) -> V {
        let mut last = Idx::min_value();
        for (i, v) in self.intervals.iter() {
            if index >= last && index < *i {
                return *v;
            }
            last = *i;
        }
        self.intervals
            .iter()
            .last()
            .expect("index out of bounds, check your implementation of the Bounded trait!")
            .1
    }
}

impl<Idx, V> IntIntervalMap<Idx, V>
where
    Idx: Bounded,
    V: PartialEq,
{
    pub fn new(value: V) -> Self {
        IntIntervalMap {
            intervals: vec![(Idx::max_value(), value)],
        }
    }

    fn defrag(&mut self) {
        let mut result = vec![];
        let drain = self.intervals.drain(..);
        for (upper, value) in drain {
            if result.last().map(|(_, v)| v) == Some(&value) {
                match result.last_mut() {
                    Some(last) => last.0 = upper,
                    None => result.push((upper, value)),
                }
            } else {
                result.push((upper, value));
            }
        }

        self.intervals = result;
    }
}

pub trait IntervalMap<Idx, V>
where
    Idx: Copy + PartialOrd,
    V: Clone + PartialEq,
{
    fn get(&self, index: Idx) -> V;

    fn assign(&mut self, range: Range<Idx>, new_value: V);

    fn assign_single(&mut self, single: Idx, value: V);
}

#[cfg(test)]
mod test {
    use crate::interval_map::*;

    #[test]
    fn map_init() {
        let map = IntIntervalMap::<u8, char>::new('a');
        assert_eq!('a', map.get(0));
        assert_eq!('a', map.get(255));
        assert_eq!('a', map.get(10));
    }

    #[test]
    fn map_single() {
        let mut map = IntIntervalMap::<u8, char>::new('a');
        map.assign(10..20, 'b');
        assert_eq!('a', map.get(0));
        assert_eq!('a', map.get(255));
        assert_eq!('b', map.get(10));
        assert_eq!('b', map.get(19));
        assert_eq!('a', map.get(20));
    }

    #[test]
    fn test_seq() {
        let mut map = IntIntervalMap::<u8, char>::new('z');
        map.assign(2..20, 'a');
        map.assign(1..5, 'a');
        map.assign(10..30, 'b');
        map.assign(11..31, 'b');
        map.assign(5..15, 'c');
        map.assign(0..30, 'a');
        map.assign(0..30, 'a');
        map.assign_single(10, '!');

        assert_eq!('!', map.get(10));
        assert_eq!('a', map.get(11));
        assert_eq!('a', map.get(0));
        assert_eq!('z', map.get(255));
        assert_eq!('b', map.get(30));
        assert_eq!('z', map.get(31));
    }
}
