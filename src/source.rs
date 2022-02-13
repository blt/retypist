// Bits taken from Martin Pool's cargo-mutants, copyright 2021 under the MIT
// license.

use crate::{mutation::Mutation, visitor::Visitor};
use anyhow::{anyhow, Context, Result};
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
    /// Path relative to the root of the tree.
    tree_relative: PathBuf,

    /// Full copy of the source.
    pub code: Rc<String>,
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile")
            .field("tree_relative", &self.tree_relative)
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
            tree_relative: tree_relative.to_owned(),
            code: Rc::new(code),
        })
    }

    /// Return the path of this file relative to the tree root, with forward slashes.
    pub fn relative_path(&self) -> &Path {
        self.tree_relative.as_path()
    }

    /// Generate a list of all mutation possibilities within this file.
    pub fn mutations(&self) -> Result<Vec<Mutation>> {
        let syn_file = syn::parse_str::<syn::File>(&self.code)?;
        let mut v = Visitor::new(self);
        v.visit_file(&syn_file);
        Ok(v.mutations)
    }

    // /// Generate a list of all mutation possibilities within this file.
    // pub fn mutations(&self) -> Result<Vec<Mutation>> {
    //     let syn_file = syn::parse_str::<syn::File>(&self.code)?;
    //     let mut v = DiscoveryVisitor::new(self);
    //     v.visit_file(&syn_file);
    //     Ok(v.mutations)
    // }

    // /// Generate a list of all mutation possibilities within this file.
    // pub fn mutations(&self) -> Result<Vec<Mutation>> {
    //     let syn_file = syn::parse_str::<syn::File>(&self.code)?;
    //     let mut v = DiscoveryVisitor::new(self);
    //     v.visit_file(&syn_file);
    //     Ok(v.mutations)
    // }

    // /// Return the path of this file relative to a given directory.
    // // TODO: Maybe let the caller do this.
    // pub fn within_dir(&self, dir: &Path) -> PathBuf {
    //     dir.join(&self.tree_relative)
    // }
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

    /// Return all the mutations that could possibly be applied to this tree.
    pub fn mutations(&self) -> Result<Vec<Mutation>> {
        let mut r = Vec::new();
        for sf in self.source_files() {
            r.extend(Rc::new(sf).mutations()?);
        }
        Ok(r)
    }
}
