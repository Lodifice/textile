use std::fmt::Debug;
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
    fn assign(&mut self, range: Range<Idx>, value: V) {
        let guessed_intervals: Vec<(Idx, V)> = shelve(&[(range.start,value), (range.end,value)],
                                                      self.intervals.clone());
        self.intervals = dedup(&range.start, &range.end, guessed_intervals);
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

fn shelve<Idx: Copy + PartialOrd, V: Copy + PartialEq>(todo: &[(Idx, V)], imap: Vec<(Idx, V)>) -> Vec<(Idx, V)> {
    match imap.as_slice() {
        [] => todo.to_vec(),
        [(l,v), rest..] => match todo {
            [(s,v1), (e,v2)] => {
                if e < l {
                    cons((*s,*v1), cons((*e,*v2), imap))
                } else if s < l {
                    cons((*s,*v1), shelve(&[(*e,*v)], imap))
                } else if s == l {
                    cons((*s,*v1), shelve(&[(*e,*v)], rest.to_vec()))
                } else {
                    cons((*l,*v), shelve(&[(*s,*v1), (*e,*v)], rest.to_vec()))
                }
            },
            [(e,v2)] => {
                if e < l {
                    cons((*e,*v2), imap)
                } else if e == l {
                    imap
                } else {
                    shelve(&[(*e,*v)], rest.to_vec())
                }
            },
            _ => unreachable!(),
        },
    }
}

fn dedup<Idx: Copy + PartialOrd, V: Copy + PartialEq>(start: &Idx, end: &Idx, imap: Vec<(Idx, V)>) -> Vec<(Idx, V)> {
    match imap.as_slice() {
        [] => imap.into(),
        [_single] => imap.into(),
        [(s1, v1), (s2, v2), rest..] => {
            if  v1 == v2 {
                dedup(start, end, cons((*s1, *v1), rest.to_vec()))
            } else {
                cons((*s1, *v1), dedup(start, end, cons((*s2, *v2), rest.to_vec())))
            }
        },
    }
}

fn cons<A>(head: A, mut tail: Vec<A>) -> Vec<A> {
    let mut new_vec = vec![head];
    new_vec.append(&mut tail);
    new_vec
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
