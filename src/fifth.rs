pub struct List<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
}

pub struct Node<T> {
    elem: T,
    next: *mut Node<T>,
}

impl<T> List<T> {
    /// Creates a new [`List`].
    pub fn new() -> Self {
        List {
            head: core::ptr::null_mut(),
            tail: core::ptr::null_mut(),
        }
    }

    /// Inserts a new element at the back of the list.
    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new_tail = Box::into_raw(Box::new(Node {
                elem,
                next: core::ptr::null_mut(),
            }));

            if self.tail.is_null() {
                // List was empty.
                self.head = new_tail;
            } else {
                (*self.tail).next = new_tail;
            }

            self.tail = new_tail;
        }
    }

    /// Removes an element from the front of the list, and returns it.
    pub fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            None
        } else {
            unsafe {
                let head = Box::from_raw(self.head);

                self.head = head.next;
                if self.head.is_null() {
                    // List became empty.
                    self.tail = core::ptr::null_mut();
                }

                Some(head.elem)
            }
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop_front(), None);

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));

        list.push_back(4);
        list.push_back(5);

        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), None);

        list.push_back(6);
        list.push_back(7);

        assert_eq!(list.pop_front(), Some(6));
        assert_eq!(list.pop_front(), Some(7));
        assert_eq!(list.pop_front(), None);
    }
}
