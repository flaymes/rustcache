extern crate failure;

#[derive(Debug, Fail)]
pub enum StorageError {
    #[fail(display = "Item expired")]
    ItemExpired,
    #[fail(display = "Key not found")]
    NotFound,
}