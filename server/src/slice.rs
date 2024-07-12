pub trait IterPlucked<T> {
    fn iter_plucked(&mut self, idx: usize) -> Option<(&mut T, std::iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>>)>;
}

impl<T> IterPlucked<T> for [T] {
    fn iter_plucked(&mut self, idx: usize) -> Option<(&mut T, std::iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>>)> {
        
        let (left_slice, element_slice_and_right_slice) = self.split_at_mut(idx);
        let (elm, right_slice) = element_slice_and_right_slice.split_first_mut()?;
        let iter = left_slice.iter().chain(right_slice.iter());

        Some((elm, iter))

    }
}
