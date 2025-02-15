use clap::Parser;

/// Align two texts on semantical similarity across 10+ languages
#[derive(Parser, Debug)]
pub struct Context {
    /// left text file
    #[arg(short, long)]
    pub left: String,

    /// right text file
    #[arg(short, long)]
    pub right: String,

    /// dir for intermediate state of alignment process and for result.json
    #[arg(short, long)]
    pub context: String,

    /// window size of alignment, affects tolerance for mismatch between files, strongly affects speed of alignment
    #[arg(short, long, default_value = "300")]
    pub window_size: usize,
    //
    // TODO: boolean flags on how to split inputs
    //
}
