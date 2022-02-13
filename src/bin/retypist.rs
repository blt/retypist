use argh::FromArgs;
use retypist::source::SourceTree;
use std::path::PathBuf;

/// Mutate a project, ideally in beneficial ways
#[derive(FromArgs, PartialEq, Debug)]
struct Args {
    /// rust crate directory to examine.
    #[argh(option, short = 'd', default = r#"PathBuf::from(".")"#)]
    dir: PathBuf,
}

fn main() {
    let args: Args = argh::from_env();
    let tree = SourceTree::new(&args.dir).unwrap();
    println!("{:?}", tree.mutations());
}
