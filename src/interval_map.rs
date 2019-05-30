use std::ops::RangeInclusive;

pub struct IntervalMap<Idx, V> {
    intervals: Vec<(Idx, V)>,
}

impl<Idx, V> IntervalMap<Idx, V>
where
    Idx: Copy + PartialOrd,
    V: Clone,
{
    pub fn assign(&mut self, range: RangeInclusive<Idx>, value: V) {
        // Find start of range
        let (split_start, old_value) = match self
            .intervals
            .iter()
            .enumerate()
            .find(|(i, (t, v))| t <= range.start())
            .map(|(i, (_, v))| (i, v))
        {
            Some(n) => n,
            None => unreachable!(),
        };

        // Remove all intervals in range
        let mut tail_intervals = vec![];
        for (idx, value) in self.intervals.drain(split_start..) {
            if idx <= *range.end() {
                continue;
            }
            tail_intervals.push((*range.end(), value));
        }

        self.intervals.push((*range.start(), old_value));
        self.intervals.push((*range.end(), v));

        self.intervals.extend(tail_intervals);
    }

    pub fn assign_single(&mut self, single: Idx, value: V) {
        self.assign(RangeInclusive::new(single, single), value);
    }
}
