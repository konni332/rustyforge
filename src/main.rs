use anyhow::Result;
use clap::Parser;
use rustyforge_core::{ForgeArgs, execute};

fn main() -> Result<()> {
    let args = ForgeArgs::parse();
    execute(args)
}
