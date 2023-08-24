/*!
 * A simple interval-tree in Rust made to store memory mappings
 */

#![no_std]
#![warn(clippy::cargo)]
#![allow(ambiguous_glob_reexports)]
#![deny(clippy::cargo_common_metadata)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::unreadable_literal,
    clippy::type_repetition_in_bounds,
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::used_underscore_binding,
    clippy::ptr_as_ptr,
    clippy::missing_panics_doc,
    clippy::missing_docs_in_private_items,
    clippy::module_name_repetitions
)]
#![cfg_attr(not(test), warn(
    missing_debug_implementations,
    //trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    //unused_results
))]
#![cfg_attr(test, deny(
    missing_debug_implementations,
    //trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_must_use,
    //unused_results
))]
#![cfg_attr(
    test,
    deny(
        bad_style,
        dead_code,
        improper_ctypes,
        non_shorthand_field_patterns,
        no_mangle_generic_items,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        private_in_public,
        unconditional_recursion,
        unused,
        unused_allocation,
        unused_comparisons,
        unused_parens,
        while_true
    )
)]
// Till they fix this buggy lint in clippy
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::borrow_deref_ref)]

#[macro_use]
pub extern crate alloc;

use alloc::boxed::Box;
use core::cmp::{Ord, Ordering};

mod node;
use node::Node;

mod interval;
pub use interval::Interval;

mod iterators;
pub use iterators::{Entry, EntryMut, IntervalTreeIterator, IntervalTreeIteratorMut};

#[derive(Clone, Debug, Default)]
pub struct IntervalTree<T: Ord + Clone, V> {
    root: Option<Box<Node<T, V>>>,
}

impl<T: Ord + Clone, V> IntervalTree<T, V> {
    #[must_use]
    pub fn new() -> Self {
        IntervalTree { root: None }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        Node::size(&self.root)
    }

    #[must_use]
    pub fn height(&self) -> i64 {
        Node::height(&self.root)
    }

    #[must_use]
    pub fn query<I: Into<Interval<T>>>(&self, interval: I) -> IntervalTreeIterator<'_, T, V> {
        if let Some(ref n) = self.root {
            IntervalTreeIterator {
                nodes: vec![n],
                interval: interval.into(),
            }
        } else {
            let nodes = vec![];
            IntervalTreeIterator {
                nodes,
                interval: interval.into(),
            }
        }
    }

    #[must_use]
    pub fn query_mut<I: Into<Interval<T>>>(
        &mut self,
        interval: I,
    ) -> IntervalTreeIteratorMut<'_, T, V> {
        if let Some(ref mut n) = self.root {
            IntervalTreeIteratorMut {
                nodes: vec![n],
                interval: interval.into(),
            }
        } else {
            let nodes = vec![];
            IntervalTreeIteratorMut {
                nodes,
                interval: interval.into(),
            }
        }
    }

    pub fn insert<I: Into<Interval<T>>>(&mut self, interval: I, value: V) {
        let interval = interval.into();
        let max = interval.end.clone();

        self.root = Some(IntervalTree::insert_helper(
            self.root.take(),
            interval,
            value,
            max,
        ));
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn insert_helper(
        node: Option<Box<Node<T, V>>>,
        interval: Interval<T>,
        value: V,
        max: T,
    ) -> Box<Node<T, V>> {
        if node.is_none() {
            return Box::new(Node::new(interval, value, max, 0, 1));
        }

        let mut node_ref = node.unwrap();

        match interval.cmp(&node_ref.interval) {
            Ordering::Less => {
                node_ref.left_child = Some(IntervalTree::insert_helper(
                    node_ref.left_child,
                    interval,
                    value,
                    max,
                ));
            }
            Ordering::Greater => {
                node_ref.right_child = Some(IntervalTree::insert_helper(
                    node_ref.right_child,
                    interval,
                    value,
                    max,
                ));
            }
            Ordering::Equal => return node_ref,
        }

        node_ref.update_height();
        node_ref.update_size();
        node_ref.update_max();

        IntervalTree::balance(node_ref)
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn balance(mut node: Box<Node<T, V>>) -> Box<Node<T, V>> {
        if Node::balance_factor(&node) < -1 {
            if Node::balance_factor(node.right_child.as_ref().unwrap()) > 0 {
                node.right_child = Some(IntervalTree::rotate_right(node.right_child.unwrap()));
            }
            node = IntervalTree::rotate_left(node);
        } else if Node::balance_factor(&node) > 1 {
            if Node::balance_factor(node.left_child.as_ref().unwrap()) < 0 {
                node.left_child = Some(IntervalTree::rotate_left(node.left_child.unwrap()));
            }
            node = IntervalTree::rotate_right(node);
        }
        node
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn rotate_right(mut node: Box<Node<T, V>>) -> Box<Node<T, V>> {
        let mut y = node.left_child.unwrap();
        node.left_child = y.right_child;
        y.size = node.size;
        node.update_height();
        node.update_size();
        node.update_max();

        y.right_child = Some(node);
        y.update_height();
        y.update_max();

        y
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn rotate_left(mut node: Box<Node<T, V>>) -> Box<Node<T, V>> {
        let mut y = node.right_child.unwrap();
        node.right_child = y.left_child;
        y.size = node.size;

        node.update_height();
        node.update_size();
        node.update_max();

        y.left_child = Some(node);
        y.update_height();
        y.update_max();

        y
    }

    pub fn delete<I: Into<Interval<T>>>(&mut self, interval: I) {
        if !self.is_empty() {
            let interval = interval.into();
            self.root = IntervalTree::delete_helper(self.root.take(), &interval);
        }
    }

    fn delete_helper(
        node: Option<Box<Node<T, V>>>,
        interval: &Interval<T>,
    ) -> Option<Box<Node<T, V>>> {
        match node {
            None => None,
            Some(mut node) => {
                if *interval < node.interval {
                    node.left_child = IntervalTree::delete_helper(node.left_child.take(), interval);
                } else if *interval > node.interval {
                    node.right_child =
                        IntervalTree::delete_helper(node.right_child.take(), interval);
                } else if node.left_child.is_none() {
                    return node.right_child;
                } else if node.right_child.is_none() {
                    return node.left_child;
                } else {
                    let mut y = node;
                    node = IntervalTree::min(&mut y.right_child);
                    node.right_child = IntervalTree::delete_min_helper(y.right_child.unwrap());
                    node.left_child = y.left_child;
                }

                node.update_height();
                node.update_size();
                node.update_max();
                Some(IntervalTree::balance(node))
            }
        }
    }

    #[allow(clippy::unnecessary_box_returns)]
    fn min(node: &mut Option<Box<Node<T, V>>>) -> Box<Node<T, V>> {
        match node {
            Some(node) => {
                if node.left_child.is_none() {
                    Box::new(Node::new(
                        node.interval.clone(),
                        node.value.take().unwrap(),
                        node.max.clone(),
                        0,
                        1,
                    ))
                } else {
                    IntervalTree::min(&mut node.left_child)
                }
            }
            None => panic!("Called min on None node"),
        }
    }

    pub fn delete_min(&mut self) {
        if !self.is_empty() {
            self.root = IntervalTree::delete_min_helper(self.root.take().unwrap());
        }
    }

    fn delete_min_helper(mut node: Box<Node<T, V>>) -> Option<Box<Node<T, V>>> {
        if node.left_child.is_none() {
            return node.right_child.take();
        }

        node.left_child = IntervalTree::delete_min_helper(node.left_child.unwrap());

        node.update_height();
        node.update_size();
        node.update_max();

        Some(IntervalTree::balance(node))
    }

    pub fn delete_max(&mut self) {
        if !self.is_empty() {
            self.root = IntervalTree::delete_max_helper(self.root.take().unwrap());
        }
    }

    fn delete_max_helper(mut node: Box<Node<T, V>>) -> Option<Box<Node<T, V>>> {
        if node.right_child.is_none() {
            return node.left_child.take();
        }

        node.right_child = IntervalTree::delete_max_helper(node.right_child.unwrap());

        node.update_height();
        node.update_size();
        node.update_max();

        Some(IntervalTree::balance(node))
    }

    pub fn clear(&mut self) {
        self.root = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_1() {
        let mut tree = IntervalTree::<usize, bool>::new();
        for i in 0..10 {
            tree.insert((i * 10)..(i * 10 + 10), false);
        }

        let mut cnt = 0;
        for _ in tree.query(0..10000) {
            cnt += 1;
        }
        assert_eq!(cnt, 10);
    }

    #[test]
    fn query_2() {
        let mut tree = IntervalTree::<usize, bool>::new();
        for i in 0..10 {
            tree.insert((i * 10)..(i * 10 + 10), false);
        }

        let mut cnt = 0;
        for _ in tree.query(0..30) {
            cnt += 1;
        }
        assert_eq!(cnt, 3);
    }
}
