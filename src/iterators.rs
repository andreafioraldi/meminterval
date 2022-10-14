use alloc::vec::Vec;
use core::{cmp::Ord, fmt::Debug};

use crate::interval::Interval;
use crate::node::Node;

#[derive(PartialEq, Eq, Debug)]
pub struct Entry<'a, T: Ord, V> {
    pub value: &'a V,
    pub interval: &'a Interval<T>,
}

pub struct IntervalTreeIterator<'a, T: Ord + Clone, V> {
    pub(crate) nodes: Vec<&'a Node<T, V>>,
    pub(crate) interval: Interval<T>,
}

impl<'a, T: Ord + Copy + 'a, V: 'a> Iterator for IntervalTreeIterator<'a, T, V> {
    type Item = Entry<'a, T, V>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let cur = match self.nodes.pop() {
                None => return None,
                Some(node) => node,
            };

            if cur.right_child.is_some() {
                self.nodes.push(cur.right_child.as_ref().unwrap());
            }
            if cur.left_child.is_some() && cur.left_child.as_ref().unwrap().max >= self.interval.end
            {
                self.nodes.push(cur.left_child.as_ref().unwrap());
            }

            if cur.interval.intersect(&self.interval).is_some() {
                return Some(Entry {
                    value: cur.value.as_ref().unwrap(),
                    interval: &cur.interval,
                });
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct EntryMut<'a, T: Ord, V> {
    pub value: &'a mut V,
    pub interval: &'a Interval<T>,
}

pub struct IntervalTreeIteratorMut<'a, T: Ord + Clone, V> {
    pub(crate) nodes: Vec<&'a mut Node<T, V>>,
    pub(crate) interval: Interval<T>,
}

impl<'a, T: Ord + Copy + 'a, V: 'a> Iterator for IntervalTreeIteratorMut<'a, T, V> {
    type Item = EntryMut<'a, T, V>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let cur = match self.nodes.pop() {
                None => return None,
                Some(node) => node,
            };

            if cur.right_child.is_some() {
                self.nodes.push(cur.right_child.as_mut().unwrap());
            }
            if cur.left_child.is_some() && cur.left_child.as_ref().unwrap().max >= self.interval.end
            {
                self.nodes.push(cur.left_child.as_mut().unwrap());
            }

            if cur.interval.intersect(&self.interval).is_some() {
                return Some(EntryMut {
                    value: cur.value.as_mut().unwrap(),
                    interval: &cur.interval,
                });
            }
        }
    }
}
