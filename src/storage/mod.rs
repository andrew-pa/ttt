use crate::model::Tree;

pub trait Storage {
    fn src_name(&self) -> String;
    fn sync(&mut self, model: &mut Tree) -> anyhow::Result<()>;
}

mod local_storage;

pub use local_storage::LocalStorage;
