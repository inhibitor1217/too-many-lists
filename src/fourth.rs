use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> List<T> {
    /// Creates an empty [`List`].
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    /// Adds a new element to the front of the list.
    pub fn push_front(&mut self, elem: T) {
        let new_node = Node::new(elem);
        if let Some(old_head) = self.head.take() {
            old_head.borrow_mut().prev = Some(new_node.clone());
            new_node.borrow_mut().next = Some(old_head);
            self.head = Some(new_node);
        } else {
            // Empty list case.
            self.tail = Some(new_node.clone());
            self.head = Some(new_node);
        }
    }

    /// Removes the first element from the list and returns it, or `None` if it is empty.
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            if let Some(new_head) = old_head.borrow_mut().next.take() {
                new_head.borrow_mut().prev = None;
                self.head = Some(new_head);
            } else {
                // List will be empty after this pop.
                self.tail.take();
            }

            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    /// Retrieves a reference to the first element of the list, or `None` if it is empty.
    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Node<T> {
    /// Creates a new node.
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            next: None,
            prev: None,
        }))
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop_front(), None);

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        list.push_front(4);
        list.push_front(5);

        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek_front() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(*list.peek_front().unwrap(), 3);
    }
}
