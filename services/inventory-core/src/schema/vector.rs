use im::Vector;

pub trait VectorAppend<T> {
    fn append(&self, item: T) -> Vector<T>;
}

impl<T: Clone> VectorAppend<T> for Vector<T> {
    fn append(&self, item: T) -> Vector<T> {
        let mut items = self.clone();
        items.push_back(item);
        items
    }
}
