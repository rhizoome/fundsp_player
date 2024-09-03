#[cfg(debug_assertions)]
use assert_no_alloc::AllocDisabler;
use clap::{Parser, Subcommand};
use runner::live;

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

fn main() {
    live("Loopback Audio", "harmonic_series");
}

mod build;
mod runner;
