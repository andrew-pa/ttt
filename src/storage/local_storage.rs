use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::model::Tree;

use super::Storage;

pub struct LocalStorage {
    path: PathBuf,
}

impl LocalStorage {
    pub fn open(path: PathBuf) -> LocalStorage {
        LocalStorage { path }
    }
}

impl Storage for LocalStorage {
    fn src_name(&self) -> String {
        self.path.to_string_lossy().into()
    }

    fn sync(&mut self, model: &mut Tree) -> Result<()> {
        // TODO: actually do a sync rather than overwrite
        let f = File::create(&self.path)?;
        ron::ser::to_writer_pretty(
            f,
            model,
            ron::ser::PrettyConfig::default().indentor("\t".into()).compact_arrays(true),
        )
        .map_err(Into::into)
    }
}
