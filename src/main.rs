use anyhow::Result;
use clap::Parser;
use rustyforge_core::ForgeArgs;

fn main() -> Result<()> {
    let args = ForgeArgs::parse();
    if args.verbose_hard {
        verbosio::set_verbosity!(2);
    } else if args.verbose {
        verbosio::set_verbosity!(1);
    }
    Ok(())
}
