use argh::FromArgs;
use retypist::{
    cargo::{run_cargo, CargoResult},
    git::run_git,
    interrupt,
    source::SourceTree,
};
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
    interrupt::install_handler();
    let tree = SourceTree::new(&args.dir).unwrap();
    loop {
        for mut mutation in tree.mutation().unwrap() {
            let change = mutation.mutate();
            let source_file = &mut mutation.source_file;
            source_file.rewrite(change).unwrap();
        }
        match run_cargo(&["check", "--tests"], tree.root()) {
            Ok(res) => match res {
                CargoResult::Success => {
                    println!("PASS");
                    run_cargo(&["fmt"], tree.root()).unwrap();
                    run_git(&["commit", "-am", "applied modifications"], tree.root()).unwrap();
                }
                CargoResult::Failure => {
                    println!("FAIL");
                    run_git(&["checkout", "."], tree.root()).unwrap();
                }
            },
            Err(err) => {
                println!("ERROR {:?}", err);
                run_git(&["checkout", "."], tree.root()).unwrap();
            }
        }
    }
}
