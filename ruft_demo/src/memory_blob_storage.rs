use ruft_core::prelude::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct MemoryBlobStorage {
    internal: HashMap<String, Vec<u8>>,
}

impl MemoryBlobStorage {
    pub fn create() -> Self {
        Self {
            internal: Default::default(),
        }
    }
}

impl BlobStorage for MemoryBlobStorage {
    fn save(&mut self, id: String, blob: Vec<u8>) -> Result<(), GenericError> {
        self.internal.insert(id, blob);
        Ok(())
    }

    fn load(&mut self, id: String) -> Result<Vec<u8>, GenericError> {
        match self.internal.remove(&id) {
            Some(buffer) => Ok(buffer),
            None => Err(GenericError::Generic("not found: todo improve".to_string())),
        }
    }
}
