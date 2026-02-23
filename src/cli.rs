use facet::Facet;

#[derive(Facet)]
#[facet(figue::version, figue::help)]
pub struct Args {
    #[facet(figue::named, default = "db.chimie")]
    pub db_path: String,

    #[facet(figue::named, figue::short = 'v')]
    pub verbose: bool,
}
