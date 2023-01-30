pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

type Link<T> = Option<Box<Node<T>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    /// Creates a new [`List`].
    pub fn new() -> Self {
        List {
            head: None,
            tail: core::ptr::null_mut(),
        }
    }

    /// Inserts a new element at the back of the list.
    pub fn push_back(&mut self, elem: T) {
        let mut new_tail = Box::new(Node { elem, next: None });
        let raw_new_tail: *mut _ = &mut *new_tail;

        if self.tail.is_null() {
            // List was empty.
            self.head = Some(new_tail);
        } else {
            unsafe {
                (*self.tail).next = Some(new_tail);
            }
        }

        self.tail = raw_new_tail;
    }

    /// Removes an element from the front of the list, and returns it.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|head| {
            self.head = head.next;

            if self.head.is_none() {
                self.tail = core::ptr::null_mut();
            }

            head.elem
        })
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
