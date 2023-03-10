use core::{fmt::Debug, hash::Hash, marker::PhantomData, mem, ptr::NonNull};

pub struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<T>,
}

struct Node<T> {
    prev: Option<NonNull<Node<T>>>,
    next: Option<NonNull<Node<T>>>,
    elem: T,
}

impl<T> LinkedList<T> {
    /// Creates a new [`LinkedList`].
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Inserts an element at the beginning of the list.
    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // Allocate the node at the heap, but we will manage the allocation.
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem,
            })));

            if let Some(old_head) = self.head {
                (*old_head.as_ptr()).prev = Some(new_node);
                (*new_node.as_ptr()).next = Some(old_head);
            } else {
                // Empty list case.
                self.tail = Some(new_node);
            }

            self.head = Some(new_node);
            self.len += 1;
        }
    }

    /// Inserts an element at the back of the list.
    pub fn push_back(&mut self, elem: T) {
        unsafe {
            // Allocate the node at the heap, but we will manage the allocation.
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem,
            })));

            if let Some(old_tail) = self.tail {
                (*old_tail.as_ptr()).next = Some(new_node);
                (*new_node.as_ptr()).prev = Some(old_tail);
            } else {
                // Empty list case.
                self.head = Some(new_node);
            }

            self.tail = Some(new_node);
            self.len += 1;
        }
    }

    /// Removes an element from the beginning of the list, and returns it.
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.head.map(|head| {
                // Reconstruct the box so that it is implicitly Dropped at the
                // end of this scope.
                let boxed_head = Box::from_raw(head.as_ptr());

                self.head = boxed_head.next;
                if let Some(new_head) = self.head {
                    (*new_head.as_ptr()).prev = None;
                } else {
                    // The list became empty.
                    self.tail = None;
                }

                self.len -= 1;
                boxed_head.elem
            })
        }
    }

    /// Removes an element from the end of the list, and returns it.
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.map(|tail| {
                // Reconstruct the box so that it is implicitly Dropped at the
                // end of this scope.
                let boxed_tail = Box::from_raw(tail.as_ptr());

                self.tail = boxed_tail.prev;
                if let Some(new_tail) = self.tail {
                    (*new_tail.as_ptr()).next = None;
                } else {
                    // The list became empty.
                    self.head = None;
                }

                self.len -= 1;
                boxed_tail.elem
            })
        }
    }

    /// Returns a reference to the first element of the list.
    pub fn front(&self) -> Option<&T> {
        unsafe { Some(&(*self.head?.as_ptr()).elem) }
    }

    /// Returns a mutable reference to the first element of the list.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { Some(&mut (*self.head?.as_ptr()).elem) }
    }

    /// Returns a reference to the last element of the list.
    pub fn back(&self) -> Option<&T> {
        unsafe { Some(&(*self.tail?.as_ptr()).elem) }
    }

    /// Returns a mutable reference to the last element of the list.
    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { Some(&mut (*self.tail?.as_ptr()).elem) }
    }

    /// The length of the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears the collection.
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        // We cannot blindly clone the pointers!
        // They are managed by the original list, and should not be shared!!
        let mut new_list = Self::new();
        for item in self {
            new_list.push_back(item.clone());
        }
        new_list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self {
            item.hash(state);
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct Iter<'a, T> {
    front: Option<NonNull<Node<T>>>,
    back: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<&'a T>,
}

impl<T> LinkedList<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            front: self.head,
            back: self.tail,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|front| unsafe {
                self.len -= 1;
                self.front = (*front.as_ptr()).next;
                &(*front.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|back| unsafe {
                self.len -= 1;
                self.back = (*back.as_ptr()).prev;
                &(*back.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IterMut<'a, T> {
    front: Option<NonNull<Node<T>>>,
    back: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<T> LinkedList<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            front: self.head,
            back: self.tail,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|front| unsafe {
                self.len -= 1;
                self.front = (*front.as_ptr()).next;
                &mut (*front.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|back| unsafe {
                self.len -= 1;
                self.back = (*back.as_ptr()).prev;
                &mut (*back.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T> LinkedList<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

pub struct CursorMut<'a, T> {
    cur: Option<NonNull<Node<T>>>,
    list: &'a mut LinkedList<T>,
    index: Option<usize>,
}

impl<T> LinkedList<T> {
    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            cur: None,
            list: self,
            index: None,
        }
    }
}

impl<'a, T> CursorMut<'a, T> {
    /// Retreive a current index of the cursor.
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Move the cursor to the next position.
    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).next;
                if self.cur.is_some() {
                    self.index = Some(self.index.unwrap() + 1);
                } else {
                    // We just moved into the ghost element.
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're at the ghost element, and there is a head element.
            self.cur = self.list.head;
            self.index = Some(0);
        } else {
            // The list is empty, do nothing.
        }
    }

    /// Move the cursor to the previous position.
    pub fn move_prev(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).prev;
                if self.cur.is_some() {
                    self.index = Some(self.index.unwrap() - 1);
                } else {
                    // We just moved into the ghost element.
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're at the ghost element, and there is a tail element.
            self.cur = self.list.tail;
            self.index = Some(self.list.len - 1);
        } else {
            // The list is empty, do nothing.
        }
    }

    /// Retrieve an element at the cursor.
    pub fn current(&mut self) -> Option<&mut T> {
        unsafe { self.cur.map(|node| &mut (*node.as_ptr()).elem) }
    }

    /// Retrieve the element next to the cursor.
    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            self.cur
                .map_or_else(|| self.list.head, |node| (*node.as_ptr()).next)
                .map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    /// Retrieve the element before the cursor.
    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            self.cur
                .map_or_else(|| self.list.tail, |node| (*node.as_ptr()).prev)
                .map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    /// Creates a new list by splitting the list before the cursor, returning the newly created list.
    /// The cursor will remain at the original list.
    pub fn split_before(&mut self) -> LinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let prev = (*cur.as_ptr()).prev;
                if let Some(prev) = prev {
                    (*prev.as_ptr()).next = None;
                    (*cur.as_ptr()).prev = None;
                }

                let splitted_list = LinkedList {
                    head: if prev.is_some() { self.list.head } else { None },
                    tail: prev,
                    len: self.index.unwrap(),
                    _marker: PhantomData,
                };

                self.list.head = Some(cur);
                self.list.len -= self.index.unwrap();
                self.index = Some(0);

                splitted_list
            }
        } else {
            // Ghost case, the original list becomes empty.
            mem::replace(self.list, LinkedList::new())
        }
    }

    /// Creates a new list by splitting the list after the cursor, returning the newly created list.
    /// The cursor will remain at the original list.
    pub fn split_after(&mut self) -> LinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let next = (*cur.as_ptr()).next;
                if let Some(next) = next {
                    (*next.as_ptr()).prev = None;
                    (*cur.as_ptr()).next = None;
                }

                let splitted_list = LinkedList {
                    head: next,
                    tail: if next.is_some() { self.list.tail } else { None },
                    len: self.list.len - self.index.unwrap() - 1,
                    _marker: PhantomData,
                };

                self.list.tail = Some(cur);
                self.list.len = self.index.unwrap() + 1;

                splitted_list
            }
        } else {
            // Ghost case, the original list becomes empty.
            mem::replace(self.list, LinkedList::new())
        }
    }

    /// Inserts the given list before the cursor.
    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        if input.is_empty() {
            return; // Do nothing if the given list is empty.
        }

        unsafe {
            if let Some(cur) = self.cur {
                let prev = (*cur.as_ptr()).prev;

                if let Some(prev) = prev {
                    (*prev.as_ptr()).next = input.head;
                    (*input.head.unwrap().as_ptr()).prev = Some(prev);
                } else {
                    self.list.head = input.head;
                }

                (*cur.as_ptr()).prev = input.tail;
                (*input.tail.unwrap().as_ptr()).next = Some(cur);
                self.list.len += input.len;
                self.index = Some(self.index.unwrap() + input.len);

                input.head = None;
                input.tail = None;
                input.len = 0;
            } else if let Some(tail) = self.list.tail {
                // Append the input list at the back of current list.
                // Cursor remains at the ghost.

                (*tail.as_ptr()).next = input.head;
                (*input.head.unwrap().as_ptr()).prev = Some(tail);

                self.list.tail = input.tail;
                self.list.len += input.len;

                input.head = None;
                input.tail = None;
                input.len = 0;
            } else {
                // We are empty, so become the input.
                // Cursor remains at the ghost.
                *self.list = input;
            }
        }
    }

    /// Inserts the given list after the cursor.
    pub fn splice_after(&mut self, mut input: LinkedList<T>) {
        if input.is_empty() {
            return; // Do nothing if the given list is empty.
        }

        unsafe {
            if let Some(cur) = self.cur {
                let next = (*cur.as_ptr()).next;

                if let Some(next) = next {
                    (*next.as_ptr()).prev = input.tail;
                    (*input.tail.unwrap().as_ptr()).next = Some(next);
                } else {
                    self.list.tail = input.tail;
                }

                (*cur.as_ptr()).next = input.head;
                (*input.head.unwrap().as_ptr()).prev = Some(cur);
                self.list.len += input.len;

                input.head = None;
                input.tail = None;
                input.len = 0;
            } else if let Some(head) = self.list.head {
                // Append the input list at the start of current list.
                // Cursor remains at the ghost.

                (*head.as_ptr()).prev = input.tail;
                (*input.tail.unwrap().as_ptr()).next = Some(head);

                self.list.head = input.head;
                self.list.len += input.len;

                input.head = None;
                input.tail = None;
                input.len = 0;
            } else {
                // We are empty, so become the input.
                // Cursor remains at the ghost.
                *self.list = input;
            }
        }
    }

    /// Removes the current element and returns it.
    /// The cursor will be moved to the next element.
    pub fn remove_current(&mut self) -> Option<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let boxed_node = Box::from_raw(cur.as_ptr());

                boxed_node
                    .prev
                    .map(|prev| (*prev.as_ptr()).next = boxed_node.next);
                boxed_node
                    .next
                    .map(|next| (*next.as_ptr()).prev = boxed_node.prev);

                if boxed_node.prev.is_none() {
                    self.list.head = boxed_node.next;
                }
                if boxed_node.next.is_none() {
                    self.list.tail = boxed_node.prev;
                }

                self.cur = boxed_node.next;
                if self.cur.is_none() {
                    self.index = None;
                }

                self.list.len -= 1;

                Some(boxed_node.elem)
                // Box is dropped and memory is freed.
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    fn generate_test() -> LinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> LinkedList<T> {
        v.iter().map(|x| (*x).clone()).collect()
    }

    #[test]
    fn basic_front() {
        let mut list = LinkedList::new();

        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn basic() {
        let mut m = LinkedList::new();
        assert_eq!(m.pop_front(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_front(), None);
        m.push_front(1);
        assert_eq!(m.pop_front(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_front(), Some(2));
        assert_eq!(m.pop_front(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_front(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_front(), Some(1));

        let mut n = LinkedList::new();
        n.push_front(2);
        n.push_front(3);
        {
            assert_eq!(n.front().unwrap(), &3);
            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn iterator_double_end() {
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn iterator_mut_double_end() {
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn eq() {
        let mut n: LinkedList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn debug() {
        let list: LinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: LinkedList<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .copied()
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn hashmap() {
        // Check that HashMap works with this as a key

        let list1: LinkedList<i32> = (0..10).collect();
        let list2: LinkedList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    fn cursor_move_peek() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));
        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_prev(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = m.cursor_mut();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_prev(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));
        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_prev(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn cursor_mut_insert() {
        let mut m: LinkedList<u32> = LinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.splice_before(Some(7).into_iter().collect());
        cursor.splice_after(Some(8).into_iter().collect());
        // check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        cursor.splice_before(Some(9).into_iter().collect());
        cursor.splice_after(Some(10).into_iter().collect());
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]
        );

        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(7));
        cursor.move_prev();
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), Some(9));
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(10));
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[1, 8, 2, 3, 4, 5, 6]
        );

        let mut cursor = m.cursor_mut();
        cursor.move_next();
        let mut p: LinkedList<u32> = LinkedList::new();
        p.extend([100, 101, 102, 103]);
        let mut q: LinkedList<u32> = LinkedList::new();
        q.extend([200, 201, 202, 203]);
        cursor.splice_after(p);
        cursor.splice_before(q);
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );
        assert_eq!(m.len(), 15);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        let tmp = cursor.split_before();
        assert_eq!(m.len(), 0);
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
        m = tmp;
        assert_eq!(m.len(), 15);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        let tmp = cursor.split_after();
        assert_eq!(tmp.len(), 8);
        assert_eq!(m.len(), 7);
        assert_eq!(
            tmp.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &LinkedList<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();

        assert_eq!(from_front, re_reved);
    }
}
