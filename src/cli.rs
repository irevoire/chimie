use std::path::PathBuf;

use facet::Facet;
use figue as args;

#[derive(Facet)]
pub struct Args {
    // That gives us the --help, --version and --completions
    #[facet(flatten)]
    pub _builtins: figue::FigueBuiltins,
    #[facet(args::named, default = "db.chimie")]
    pub db_path: PathBuf,

    #[facet(args::named, args::short = 'v')]
    pub verbose: bool,
}
