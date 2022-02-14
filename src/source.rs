// Bits taken from Martin Pool's cargo-mutants, copyright 2021 under the MIT
// license.

use crate::{mutation::Mutation, visitor::Visitor};
use anyhow::{anyhow, Context, Result};
use rand::prelude::SliceRandom;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::fmt;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use syn::visit::Visit;

/// A Rust source file within a source tree.
///
/// It can be viewed either relative to the source tree (for display)
/// or as a path that can be opened (relative to cwd or absolute.)
///
/// Code is normalized to Unix line endings as it's read in, and modified
/// files are written with Unix line endings.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path of the file, including the user passed working directory
    path: PathBuf,

    /// Full copy of the source.
    pub code: Rc<String>,
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile")
            .field("path", &self.path)
            .finish()
    }
}

impl SourceFile {
    /// Construct a SourceFile representing a file within a tree.
    ///
    /// This eagerly loads the text of the file.
    pub fn new(tree_path: &Path, tree_relative: &Path) -> Result<SourceFile> {
        let full_path = tree_path.join(tree_relative);
        let code = std::fs::read_to_string(&full_path)
            .with_context(|| format!("failed to read source of {:?}", full_path))?
            .replace("\r\n", "\n");
        Ok(SourceFile {
            path: full_path,
            code: Rc::new(code),
        })
    }

    /// Generate a list of all mutation possibilities within this file.
    pub fn mutations(&self) -> Result<Vec<Mutation>> {
        let syn_file = syn::parse_str::<syn::File>(&self.code)?;
        let mut v = Visitor::new(self);
        v.visit_file(&syn_file);
        Ok(v.mutations)
    }

    pub fn rewrite(&mut self, edit: String) -> Result<()> {
        std::fs::write(&self.path, &edit).map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub struct SourceTree {
    root: PathBuf,
}

impl SourceTree {
    pub fn new(root: &Path) -> Result<SourceTree> {
        if !root.join("Cargo.toml").is_file() {
            return Err(anyhow!(
                "{} does not contain a Cargo.toml: specify a crate directory",
                root.to_str().unwrap(),
            ));
        }
        Ok(SourceTree {
            root: root.to_owned(),
        })
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    /// Return an iterator of `src/**/*.rs` paths relative to the root.
    pub fn source_files(&self) -> impl Iterator<Item = SourceFile> + '_ {
        walkdir::WalkDir::new(self.root.join("src"))
            .sort_by_file_name()
            .into_iter()
            .filter_map(|r| {
                r.map_err(|err| eprintln!("error walking source tree: {:?}", err))
                    .ok()
            })
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| {
                path.extension()
                    .map_or(false, |p| p.eq_ignore_ascii_case("rs"))
            })
            .filter_map(move |full_path| {
                let tree_relative = full_path.strip_prefix(&self.root).unwrap();
                SourceFile::new(&self.root, tree_relative)
                    .map_err(|err| {
                        eprintln!(
                            "error reading source {}: {}",
                            full_path.to_str().unwrap(),
                            err
                        );
                    })
                    .ok()
            })
    }

    /// Return possible mutations for the tree.
    pub fn mutation(&self) -> Result<Vec<Mutation>> {
        let mut rng = rand::thread_rng();
        let total: usize = rng.gen_range(1..16);
        let mut mutations = Vec::with_capacity(total);
        // TODO add a timeout for search here or something, could loop forever
        while mutations.len() < total {
            let sf = self.source_files().choose(&mut rng);
            if let Some(sf) = sf {
                let mut possible_mutants = sf.mutations()?;
                if possible_mutants.is_empty() {
                    continue;
                }
                possible_mutants.shuffle(&mut rng);
                mutations.push(possible_mutants.pop().unwrap());
            }
        }
        Ok(mutations)
    }
}
