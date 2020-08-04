use evc::OperationCache;

#[derive(Clone, Debug, Default)]
pub struct EVec<T>(pub Vec<T>);


impl<T> EVec<T> {
    pub fn with_compacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operation<T> {
    Push(T),
    Remove(usize),
    Clear,
}

impl<T> OperationCache for EVec<T> where T: Clone {
    type Operation = Operation<T>;

    fn apply_operation(&mut self, operation: Self::Operation) {
        match operation {
            Operation::Push(value) => self.0.push(value),
            Operation::Remove(index) => { self.0.remove(index); },
            Operation::Clear => self.0.clear(),
        }
    }
}

// fn example() {
//     let (mut w_handle, r_handle) = evc::new(EVec::default());

//     w_handle.write(Operation::Push(42));
//     w_handle.write(Operation::Push(24));

//     assert_eq!(r_handle.read().0, &[]);

//     w_handle.refresh();

//     assert_eq!(r_handle.read().0, &[42, 24]);

//     w_handle.write(Operation::Push(55));
//     w_handle.write(Operation::Remove(0));
//     w_handle.refresh();

//     assert_eq!(r_handle.read().0, &[24, 55]);

//     w_handle.write(Operation::Clear);

//     assert_eq!(r_handle.read().0, &[24, 55]);

//     w_handle.refresh();

//     assert_eq!(r_handle.read().0, &[]);
// }

