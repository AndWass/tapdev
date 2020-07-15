use crate::dataframer;

#[derive(Clone)]
pub struct Framer {
    data: Vec<u8>,
    next_stuffed: bool,
    started: bool,
    done: bool,
}

impl Framer {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next_stuffed: false,
            started: false,
            done: false,
        }
    }

    pub fn reset(&mut self) {
        self.data.clear();
        self.next_stuffed = false;
        self.started = false;
        self.done = false;
    }

    fn add_byte(&mut self, byte: u8) {
        match byte {
            0x02 => {
                self.reset();
                self.started = true;
            }
            0x03 => self.done = self.started,
            0x04 => self.next_stuffed = self.started,
            x => {
                if self.started {
                    if self.next_stuffed {
                        self.data.push(!x);
                    } else {
                        self.data.push(x);
                    }
                    self.next_stuffed = false;
                }
            }
        };
    }
}

impl dataframer::Framer for Framer {
    fn add_slice(&mut self, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut retval = Vec::new();
        for byte in bytes {
            self.add_byte(*byte);
            if self.done {
                retval.push(std::mem::replace(&mut self.data, Vec::new()));
                self.reset();
            }
        }
        retval
    }

    fn frame_data(bytes: &[u8]) -> Vec<u8> {
        let mut retval = vec![0x02u8];
        for byte in bytes {
            match *byte {
                0x02 | 0x03 | 0x04 => {
                    retval.push(0x04);
                    retval.push(!*byte);
                }
                b => {
                    retval.push(b);
                }
            };
        }
        retval.push(0x03);
        retval
    }
}
