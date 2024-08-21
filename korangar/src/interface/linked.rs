use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LinkedElementInner {
    Set(usize),
    Hidden,
    Unset,
}

pub struct LinkedElement {
    inner: Cell<LinkedElementInner>,
}

impl LinkedElement {
    pub fn new() -> Self {
        Self {
            inner: Cell::new(LinkedElementInner::Unset),
        }
    }

    pub fn link<T: ?Sized>(&self, element: &Rc<T>) {
        let element_address = Rc::<T>::as_ptr(element) as *const () as usize;
        self.inner.set(LinkedElementInner::Set(element_address));
    }

    pub fn link_hidden(&self) {
        self.inner.set(LinkedElementInner::Hidden);
    }

    pub fn is_linked(&self) -> bool {
        self.inner.get() != LinkedElementInner::Unset
    }

    pub fn is_hidden(&self) -> bool {
        self.inner.get() == LinkedElementInner::Hidden
    }

    pub fn is_linked_to<T: ?Sized>(&self, element: &Rc<T>) -> bool {
        let element_address = Rc::<T>::as_ptr(element) as *const () as usize;
        self.inner.get() == LinkedElementInner::Set(element_address)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Guard against unintended size increase.
    #[test]
    fn linked_element_size() {
        assert_eq!(16, size_of::<LinkedElement>());
    }

    #[test]
    fn default_is_unlinked_and_unhidden() {
        let element = LinkedElement::new();

        assert!(!element.is_linked());
        assert!(!element.is_hidden());
    }

    #[test]
    fn elements_can_be_linked() {
        let parent = Rc::new("parent");

        let element = LinkedElement::new();
        element.link(&parent);

        assert!(element.is_linked());
        assert!(!element.is_hidden());
        assert!(element.is_linked_to(&parent));
    }

    #[test]
    fn links_can_be_distinguished() {
        let parent = Rc::new("parent");
        let stranger = Rc::new("stranger");

        let element = LinkedElement::new();
        element.link(&parent);

        assert!(element.is_linked());
        assert!(!element.is_hidden());
        assert!(!element.is_linked_to(&stranger));
    }

    #[test]
    fn links_can_be_hidden() {
        let element = LinkedElement::new();

        element.link_hidden();

        assert!(element.is_linked());
        assert!(element.is_hidden());
    }
}
