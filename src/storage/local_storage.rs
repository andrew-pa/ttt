use std::{fs::File, path::PathBuf};

use anyhow::Result;

use crate::model::Tree;

use super::Storage;

pub struct LocalStorage {
    path: PathBuf,
}

impl LocalStorage {
    pub fn new(path: PathBuf) -> LocalStorage {
        LocalStorage { path }
    }
}

impl Storage for LocalStorage {
    fn src_name(&self) -> String {
        self.path.to_string_lossy().into()
    }

    fn load(&mut self) -> Result<Option<Tree>> {
        let f = match File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        ron::de::from_reader(f).map(Some).map_err(Into::into)
    }

    fn sync(&mut self, model: &mut Tree) -> Result<()> {
        // TODO: actually do a sync rather than overwrite
        let f = File::create(&self.path)?;
        ron::ser::to_writer_pretty(
            f,
            model,
            ron::ser::PrettyConfig::default()
                .indentor("\t".into())
                .compact_arrays(true),
        )
        .map_err(Into::into)
    }
}
