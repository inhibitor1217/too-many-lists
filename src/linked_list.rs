use core::{fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};

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
        while let Some(_) = self.pop_front() {}
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

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
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
        while let Some(_) = self.pop_front() {}
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

#[cfg(test)]
mod test {
    use super::LinkedList;

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
}
