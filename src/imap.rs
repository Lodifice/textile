use std::fmt::Debug;
use std::ops::Range;

use gen_iter::GenIter;
use num::{Bounded, Num};

#[derive(Debug, Clone, PartialEq)]
pub struct IntIntervalMap<Idx, V> {
    intervals: Vec<(Idx, V)>,
}

impl<Idx, V> IntervalMap<Idx, V> for IntIntervalMap<Idx, V>
where
    Idx: Copy + PartialOrd + Num + Bounded + Debug,
    V: Copy + PartialEq + Debug,
{
    fn assign(&mut self, range: Range<Idx>, value: V) {
        let guessed_intervals: Vec<(Idx, V)> = shelve(range, value, &mut self.intervals).collect();
        self.intervals = Dedup::new(|(s1,v1), (s2,v2)| v1 == v2, guessed_intervals.into_iter()).collect();
    }

    fn assign_single(&mut self, single: Idx, value: V) {
        self.assign(single..single + Idx::one(), value);
    }

    fn get(&self, index: Idx) -> V {
        let mut val = match self.intervals.iter().next() {
            Some((i, v)) => *v,
            None => unreachable!(),
        };
                      
        for (i, v) in self.intervals.iter() {
            if index < *i {
                return val;
            }
            val = *v;
        }

        match self.intervals.iter().last() {
            Some((i, v)) => *v,
            None => unreachable!(),
        }
    }
}

impl<Idx, V> IntIntervalMap<Idx, V>
where
    Idx: Bounded,
    V: PartialEq,
{
    pub fn debug(&self)
    where
        V: Debug,
        Idx: Debug,
    {
        for (start, value) in &self.intervals {
            eprintln!("{:?}: {:?}", start, value);
        }
    }

    pub fn new(value: V) -> Self {
        IntIntervalMap {
            intervals: vec![(Idx::min_value(), value)],
        }
    }
}

fn shelve<'a, Idx: Copy + PartialOrd, V: Copy + PartialEq>(range: Range<Idx>, value: V, imap: &'a mut Vec<(Idx, V)>) -> impl Iterator<Item=(Idx, V)> + 'a {
    GenIter(move || {
        let mut start = Some((range.start, value));
        let mut end = Some((range.end, value));

        while let Some((l, v)) = imap.first() {
            match (start, end) {
                (Some((s,v1)), Some((e,v2))) => {
                    if e < *l {
                        yield (s, v1);
                        yield (e, v2);
                        start = None;
                        end = None;
                    } else if s < *l {
                        yield (s, v1);
                        end = Some((e, *v));
                        start = None;
                    } else if s == *l {
                        yield (s, v1);
                        end = Some((e, *v));
                        start = None;
                        imap.remove(0);
                    } else {
                        yield (*l, *v);
                        start = Some((s, v1));
                        end = Some((e, *v));
                        imap.remove(0);
                    }
                },
                (None, Some((e,v2))) => {
                    if e < *l {
                        yield (e, v2);
                        end = None;
                    } else if e == *l {
                        end = None;
                    } else {
                        end = Some((e, *v));
                        imap.remove(0);
                    }
                },
                (None, None) => {
                    yield (*l, *v);
                    imap.remove(0);
                },
                _ => unreachable!()
            }
        }

        if let Some((l, v)) = start {
            yield (l, v);
        }

        if let Some((l, v)) = end {
            yield (l, v);
        }
    })
}

struct Dedup<A, I> {
    eqls: fn(&A, &A) -> bool,
    last: Option<A>,
    iter: I,
}

impl<A: Clone, I: Iterator<Item=A>> Dedup<A, I> {
    fn new(eqls: fn(&A, &A) -> bool, mut iter: I) -> Dedup<A, I> {
        Dedup { eqls: eqls, last: iter.next().map(|x| x.clone()), iter }
    }
}

impl<A: Clone, I: Iterator<Item=A>> Iterator for Dedup<A, I> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.last.clone();
        loop {
            match self.iter.next() {
                Some(a) => {
                    if !(self.eqls)(&a, self.last.as_ref().unwrap()) {
                        self.last = Some(a);
                        break;
                    }
                    self.last = Some(a);
                },
                None => {
                    self.last = None;
                    break;
                }
            }
        }
        ret
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
    use crate::imap::*;

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
