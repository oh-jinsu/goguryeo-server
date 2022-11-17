pub trait Bytes {
    fn truncate_last(&self) -> &Self;

    fn to_sized(&self, size: usize) -> Vec<u8>;

    fn clone_into_array<T>(&self) -> T where T: Sized + Default + AsMut<[u8]>;
}

impl Bytes for [u8] {
    fn truncate_last(&self) -> &Self {
        for i in 0..self.len() {
            let i = self.len() - 1 - i;
            
            if self[i] != 0 {
                return &self[..=i]
            }
        }

        &self
    }

    fn to_sized(&self, size: usize) -> Vec<u8> {
        let mut result = vec![];

        for i in 0..size {
            if i < self.len() {
                result.push(self[i]);
            } else {
                result.push(0)
            }
        }

        result
    }

    fn clone_into_array<T>(&self) -> T where T: Sized + Default + AsMut<[u8]> {
        let mut result = Default::default();

        <T as AsMut<[u8]>>::as_mut(&mut result).clone_from_slice(self);
        
        result
    }
}
