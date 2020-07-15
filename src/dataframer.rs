pub trait Framer: Clone {
    fn add_slice(&mut self, bytes: &[u8]) -> Vec<Vec<u8>>;
    fn frame_data(bytes: &[u8]) -> Vec<u8>;
}

#[derive(Copy, Clone)]
pub struct Identity {}

impl Framer for Identity {
    fn add_slice(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        vec![(Vec::from(bytes))]
    }

    fn frame_data(bytes: &[u8]) -> Vec<u8> {
        Vec::from(bytes)
    }
}
