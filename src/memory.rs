use std::{
    sync::{Arc, Mutex},
    ops::{Deref, DerefMut}, borrow::{BorrowMut, Borrow},
};

#[derive(Debug)]
pub struct Rcc<T: Copy> {
    data: Arc<Mutex<T>>,
}

impl<T: Copy> Rcc<T> {
    pub fn new(data: T) -> Self {
        Rcc {
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn from_other(other: &Rcc<T>) -> Self {
        Rcc {
            data: other.data.clone()
        }
    }
}

impl<T: Copy> Clone for Rcc<T> {
    fn clone(&self) -> Self {
        Rcc::from_other(self)
    }
}

impl<T: Copy> Deref for Rcc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.lock().unwrap().deref()
    }
}

impl<T: Copy> DerefMut for Rcc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.lock().unwrap().deref_mut()
    }
}

impl<T: Copy> Borrow<T> for Rcc<T> {
    fn borrow(&self) -> &T {
        self.data.lock().unwrap().deref()
    }
}

impl<T: Copy> BorrowMut<T> for Rcc<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.data.lock().unwrap().deref_mut()
    }
}

