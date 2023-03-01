use std::{
    rc::Rc,
    cell::RefCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
struct Rcc<T: Copy> {
    data: Rc<RefCell<T>>,
}

impl<T: Copy> Rcc<T> {
    pub fn new(data: T) -> Self {
        Rcc {
            data: Rc::new(RefCell::new(data)),
        }
    }

    pub fn from_other(other: &Rcc<T>) -> Self {
        Rcc {
            data: other.data.clone()
        }
    }
}
 
impl<T: Copy> Deref for Rcc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.data.as_ptr().as_ref()
        }.unwrap() 
    }
}

impl<T: Copy> DerefMut for Rcc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            (*self.data).as_ptr().as_mut()
        }.unwrap()
    }
}
