use std::{collections::BinaryHeap, cmp::Ordering};

/// Sequential iterator iterates over 1 or more iterators
/// It consumes in the order of adding. After one exhausted, it 
/// moves to next, until all iterators had been exhausted.
/// 
/// Example
/// 
/// ```
/// use iterutils::SeqIter;
/// let v1 = vec![1,2,3,4,5]; // first iterator
/// let v2 = vec![6,7,8]; // second iterator
/// let v3 = vec![9,10]; // third iterator
/// let v4 = [1,2,4,5,6,7,8]; // last iterator. Iterator can be of differen type
/// let mut seq_iter = SeqIter::new(); // create new SeqIterator
/// seq_iter.add(Box::new(v1.into_iter())); // SeqIter must take ownership
/// seq_iter.add(Box::new(v2.into_iter())); // Add more items
/// seq_iter.add(Box::new(v3.into_iter())); // more to add
/// seq_iter.add(Box::new(v4.into_iter())); // different iter type can be mixed
/// for i in seq_iter { // iterate over the iterators
///     println!("{i}");
/// }
/// 
/// ```
pub struct SeqIter<T> {
    ptr: usize,
    iters: Vec<Box<dyn Iterator<Item = T>>>,
}

impl<T> SeqIter<T> {
    // Create an empty SeqIter.
    pub fn new() -> SeqIter<T> {
        return SeqIter { ptr: 0, iters: Vec::new() }
    }

    // Add more Boxed iterator into the sequential iterator
    pub fn add(&mut self, iter: Box<dyn Iterator<Item=T>>) {
        self.iters.push(iter);
    }

    fn get_current(&mut self) -> Option<&mut Box<dyn Iterator<Item = T>>> {
        return self.iters.get_mut(self.ptr);
    }
}

/// Implementation for Iterator
impl<T> Iterator for SeqIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let target = self.get_current();
        if target.is_none() {
            return None
        }

        let next = target.unwrap().next();
        if next.is_some() {
            return next;
        }

        self.ptr += 1;
        return self.next();
    }
}


/// A flexible multi iterator
/// It allow you to use a choose function to choose item to iterator
/// 
/// An iterator may contain the following iterators like the following:
/// [1,2,3,4,5]
/// [3,2,1,4,7]
/// [5,6]
/// 
/// This Iterator will keep track the head elements and use your choose function to
/// choose next element and advance the corresponding iterator.
/// 
/// For example, if your iterator prefer the first odd number, if not found, first even number, then
/// the actual iterator will return
/// 
/// Choose from [1,3,5] -> 1 and advance first iterator
/// Choose from [2,3,5] -> 3 and advance second iterator
/// Choose from [2,2,5] -> 5 and advance third iterator
/// Choose from [2,2,6] -> 2 and advance first iterator
/// Choose from [3,2,6] -> 3 and advance first iterator
/// Choose from [4,2,6] -> 4 and advance first iterator
/// ...
/// 
/// Example
/// 
/// ```
/// use iterutils::MultiIterator;
/// let v1: Vec<i32> = vec![1,2,3,4,5,11,19];
/// let v2: Vec<i32> = vec![1,7,8,12,44,231];
/// let v3: Vec<i32> = vec![3,5,7,9,10,1000];
/// let v4: [i32; 7] = [1,2,4,5,6,7,8];
/// let choose_fn = |x:&Vec<i32>| -> Option<usize> {
///     let result = x.iter().enumerate().min_by(|x, y| {
///         x.1.cmp(&y.1)
///     }).map(|x| x.0);
///     println!("Choosing from {:?} -> {:?}", x, result);
///     return result;
/// };
/// // The choose function choses smallest (not very efficient though)
/// let mut min_iter = MultiIterator::new(choose_fn);
/// min_iter.add(Box::new(v1.into_iter()));
/// min_iter.add(Box::new(v2.into_iter()));
/// min_iter.add(Box::new(v3.into_iter()));
/// min_iter.add(Box::new(v4.into_iter()));
/// for i in min_iter{
///     println!("{i}");
/// }
/// 
/// // Numbers will be printed in sorted order.
/// ```
///
pub struct MultiIterator<T> {
    head: Vec<T>,
    iters: Vec<Box<dyn Iterator<Item = T>>>, 
    choose_function: fn(&Vec<T>)->Option<usize>
}

impl<T> MultiIterator<T> {
    /// Create an empty MultiIterator with choose function
    /// When choose function returns None, the iterator ends.
    /// 
    /// The choose function chooses the an index from the head elements from the iterators.
    pub fn new(choose_function:fn(&Vec<T>)->Option<usize>) -> MultiIterator<T> {
        MultiIterator {
            head:vec!(),
            iters: vec!(),
            choose_function
        }
    }

    /// Add new iterator to the list.
    /// It does not affect elements already emitted.
    pub fn add(&mut self, iter:Box<dyn Iterator<Item=T>>) {
        let mut iter = iter;
        let head = iter.next();
        if head.is_some() {
            self.head.push(head.unwrap());
            self.iters.push(iter);
        }
    }

    fn choose(&mut self)->Option<T> {
        if self.head.len() == 0 {
            return None;
        }
        // Check selected index. If the index is not selected, return end of iterator
        let index = (self.choose_function)(&self.head);
        if index.is_none() {
            return None;
        }
        // Find the index
        let index = index.unwrap();

        // If the index is invalid, return None
        if index > self.head.len() - 1 {
            return None;
        }

        // Get the next item at the same location
        let iter = self.iters.get_mut(index);
        let iter_next = iter.unwrap().next();
        if iter_next.is_none() {
            let _ = self.iters.remove(index);
            let removed = self.head.remove(index);
            
            // return head[index]
            return Some(removed);
        } else {
            // Swap the result in
            let mut next_elem = iter_next.unwrap();
            std::mem::swap(&mut self.head[index], &mut next_elem);
            return Some(next_elem);
        }
    }
}


/// Iterator implementation for MultiIterator
impl<T> Iterator for MultiIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        return self.choose();
    }
}


/// A special ordered iterator that helps you to iterate elements with global 
/// order. 
/// 
/// An example usage is that you have 25 sorted iterators loaded from file, they are huge.
/// You want to merge them into a large file that is sorted globally in ascending or descending order.
/// 
/// Example
/// 
/// ```
/// use iterutils::OrderedIterator;
/// let v1 = vec![1,2,3,4,5,11,19];
/// let v2 = vec![1,7,8,12,44,231];
/// let v3 = vec![3,5,7,9,10,1000];
/// let v4 = [1,2,4,5,6,7,8];
/// let mut o_iter: OrderedIterator<_> = OrderedIterator::new_min();
/// o_iter.add(Box::new(v1.into_iter()));
/// o_iter.add(Box::new(v2.into_iter()));
/// o_iter.add(Box::new(v3.into_iter()));
/// o_iter.add(Box::new(v4.into_iter()));
/// for i in o_iter{
///     println!("{i}");
/// }
/// ```
/// 
/// `OrderedIterator` only support `Ord` items.
/// 
/// Internally it uses min/max heap to select. This is more efficient than MultiIterator typically.
/// But MultiIterator can achieve something this iterator can't achieve.
pub struct OrderedIterator<T> 
    where T:Ord
{
    comparator: fn(&T, &T) -> Ordering,
    head: BinaryHeap<HeapItem<T>>,
    iters: Vec<Box<dyn Iterator<Item = T>>>, 
}

struct HeapItem<T> {
    what:T,
    iter_index: usize,
    comparator: fn(&T,&T)->std::cmp::Ordering
}

impl<T> Ord for HeapItem<T> {
    fn cmp(&self, other:&Self) -> Ordering {
        (self.comparator)(&self.what, &other.what)
    }
}

impl<T> PartialEq for HeapItem<T> {
    fn eq(&self, other: &Self) -> bool {
        (self.comparator)(&self.what, &other.what) == Ordering::Equal
    }
}

impl<T> Eq for HeapItem<T> {
}

impl<T> PartialOrd for HeapItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Iterator implementation for OrderedIterator
impl<T> Iterator for OrderedIterator<T> 
    where T:Ord
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.choose()
    }
}
impl<T> OrderedIterator<T> 
    where T:Ord
{
    /// Create a new min iterator that iterates item from small to large
    pub fn new_min() -> OrderedIterator<T> {
        let comparator = |x:&T, y:&T| {
            y.cmp(&x)
        };
        OrderedIterator {
            comparator,
            head: BinaryHeap::new(),
            iters: vec!(),
        }
    }

    /// Create new iterator that iterators elements from large to small
    pub fn new_max() -> OrderedIterator<T> {
        let comparator = |x:&T, y:&T| {
            x.cmp(&y)
        };
        OrderedIterator {
            comparator,
            head: BinaryHeap::new(),
            iters: vec!(),
        }
    }

    /// You should only add ordered iterator (e.g. sort before adding.)
    /// 
    /// For min iterator, sort elements in Ascending before adding
    /// For max iterator, sort elements in Descending before adding
    pub fn add(&mut self, iter:Box<dyn Iterator<Item=T>>) {
        let mut iter = iter;
        let head = iter.next();
        if head.is_some() {
            let head = head.unwrap();
            let item = HeapItem {
                what: head,
                comparator: self.comparator,
                iter_index: self.iters.len(),
            };
            self.head.push(item);
            self.iters.push(iter);
        }
    }

    fn choose(&mut self)->Option<T> {
        if self.head.len() == 0 {
            return None;
        }
        // Check selected index. If the index is not selected, return end of iterator
        let chosen = self.head.pop();
        if chosen.is_none() {
            return None;
        }
        // Find the index
        let chosen = chosen.unwrap();
        let chosen_index = chosen.iter_index;
        // Get the next item at the same location
        let iter = self.iters.get_mut(chosen_index);
        let iter_next = iter.unwrap().next();
        if iter_next.is_some() {
            let next_elem = iter_next.unwrap();
            self.head.push(HeapItem {
                comparator: self.comparator,
                iter_index: chosen_index,
                what: next_elem
            });
        }
        return Some(chosen.what);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seq_test() {
        let v1 = vec![1,2,3,4,5];
        let v2 = vec![6,7,8];
        let v3 = vec![9,10];
        let v4 = [1,2,4,5,6,7,8];
        let mut seq_iter = SeqIter::new();
        seq_iter.add(Box::new(v1.into_iter()));
        seq_iter.add(Box::new(v2.into_iter()));
        seq_iter.add(Box::new(v3.into_iter()));
        seq_iter.add(Box::new(v4.into_iter()));

        for i in seq_iter {
            println!("{i}");
        }

    }


    #[test]
    fn test_multi_1() {
        let v1: Vec<i32> = vec![1,2,3,4,5,11,19];
        let v2: Vec<i32> = vec![1,7,8,12,44,231];
        let v3: Vec<i32> = vec![3,5,7,9,10,1000];
        let v4: [i32; 7] = [1,2,4,5,6,7,8];


        let choose_fn = |x:&Vec<i32>| -> Option<usize> {
            let result = x.iter().enumerate().min_by(|x, y| {
                x.1.cmp(&y.1)
            }).map(|x| x.0);
            println!("Choosing from {:?} -> {:?}", x, result);
            return result;
        };

        let mut min_iter = MultiIterator::new(choose_fn);
        min_iter.add(Box::new(v1.into_iter()));
        min_iter.add(Box::new(v2.into_iter()));
        min_iter.add(Box::new(v3.into_iter()));
        min_iter.add(Box::new(v4.into_iter()));

        for i in min_iter{
            println!("{i}");
        }


    }


    #[test]
    fn test_ordered() {
        let v1 = vec![1,2,3,4,5,11,19];
        let v2 = vec![1,7,8,12,44,231];
        let v3 = vec![3,5,7,9,10,1000];
        let v4 = [1,2,4,5,6,7,8];

        let mut o_iter: OrderedIterator<_> = OrderedIterator::new_min();
        o_iter.add(Box::new(v1.into_iter()));
        o_iter.add(Box::new(v2.into_iter()));
        o_iter.add(Box::new(v3.into_iter()));
        o_iter.add(Box::new(v4.into_iter()));

        for i in o_iter{
            println!("{i}");
        }

        let v1 = vec!(9,4,2);
        let v2:Vec<i32> = vec!();
        let v3 = vec!(999,22,3,1);
        let v4 = [992,222,211,20,19,2,1];

        let mut o_iter = OrderedIterator::new_max();
        o_iter.add(Box::new(v1.into_iter()));
        o_iter.add(Box::new(v2.into_iter()));
        o_iter.add(Box::new(v3.into_iter()));
        o_iter.add(Box::new(v4.into_iter()));
        for i in o_iter {
            println!("{i}");
        }
    }
}
