use crate::model::Tree;
use anyhow::Result;

pub trait Storage {
    fn src_name(&self) -> String;
    fn load(&mut self) -> Result<Option<Tree>>;
    fn sync(&mut self, model: &mut Tree) -> Result<()>;
}

mod local_storage;

pub use local_storage::LocalStorage;
use url::Url;

pub fn open_storage(url_or_path: &str) -> Result<(Option<Tree>, Box<dyn Storage>)> {
    if url_or_path.starts_with('.') || url_or_path.starts_with('~') {
        let mut ns = crate::storage::LocalStorage::new(url_or_path.into());
        Ok((ns.load()?, Box::new(ns)))
    } else {
        let url = Url::parse(url_or_path)?;
        if url.scheme() == "file" {
            let mut ns = crate::storage::LocalStorage::new(url.path().into());
            Ok((ns.load()?, Box::new(ns)))
        } else {
            anyhow::bail!("unimplemented URL scheme {}", url.scheme());
        }
    }
}
