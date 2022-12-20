use crate::model::Tree;
use anyhow::Result;

pub trait Storage {
    fn src_name(&self) -> String;
    fn load(&mut self) -> Result<Option<Tree>>;
    fn sync(&mut self, model: &mut Tree) -> Result<()>;
}

mod local_storage;

pub use local_storage::LocalStorage;
