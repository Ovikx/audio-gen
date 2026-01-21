use std::{cell::RefCell, io::Error, rc::Rc};

const INPUT_BUFFER_ROW_SIZE: usize = 1 << 8;

pub struct ExternalInputBuffer {
    pub bool: Vec<bool>,
    pub f32: Vec<f32>,
    pub u32: Vec<u32>,
}

pub type SharedExternalInputBuffer = Rc<RefCell<ExternalInputBuffer>>;

impl ExternalInputBuffer {
    pub fn new(buffer_size: usize) -> Self {
        ExternalInputBuffer {
            bool: vec![false; buffer_size],
            f32: vec![0.; buffer_size],
            u32: vec![0; buffer_size],
        }
    }

    pub fn new_shared(buffer_size: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(ExternalInputBuffer::new(buffer_size)))
    }

    pub fn update_bool(&mut self, index: usize, new_value: bool) -> Result<(), Error> {
        if index >= INPUT_BUFFER_ROW_SIZE {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "requested index {} is out of bounds for buffer size {}",
                    index, INPUT_BUFFER_ROW_SIZE
                ),
            ));
        }

        self.bool[index] = new_value;
        Ok(())
    }

    pub fn update_f32(&mut self, index: usize, new_value: f32) -> Result<(), Error> {
        if index >= INPUT_BUFFER_ROW_SIZE {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "requested index {} is out of bounds for buffer size {}",
                    index, INPUT_BUFFER_ROW_SIZE
                ),
            ));
        }

        self.f32[index] = new_value;
        Ok(())
    }

    pub fn update_u32(&mut self, index: usize, new_value: u32) -> Result<(), Error> {
        if index >= INPUT_BUFFER_ROW_SIZE {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "requested index {} is out of bounds for buffer size {}",
                    index, INPUT_BUFFER_ROW_SIZE
                ),
            ));
        }

        self.u32[index] = new_value;
        Ok(())
    }
}
