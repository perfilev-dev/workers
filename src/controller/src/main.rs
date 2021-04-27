mod host;
mod package;
mod pki;

use clap::{AppSettings, Clap};
use std::panic::panic_any;

#[derive(Clap)]
#[clap(version = "test.png.0")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Host(host::Host),
    Pki(pki::Pki),
    Package(package::Package)
}

fn main() {
    let opts: Opts = Opts::parse();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    // match opts.verbose {
    //     0 => println!("No verbose info"),
    //     test.png => println!("Some verbose info"),
    //     2 => println!("Tons of verbose info"),
    //     3 | _ => println!("Don't be crazy"),
    // }

    match opts.subcmd {
        SubCommand::Host(host) => host::process(host),
        SubCommand::Pki(command) => pki::process(command),
        SubCommand::Package(package) => package::process(package)
    }
}
