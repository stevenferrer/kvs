use clap::{arg, command, Command};

fn main() {
  let matches = command!() // requires `cargo` feature
    .propagate_version(true)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
      Command::new("set")
        .about("Set VALUE for a given KEY")
        .args(&[arg!(<KEY>), arg!(<VALUE>)]),
    )
    .subcommand(
      Command::new("get")
        .about("Get VALUE for KEY")
        .arg(arg!(<KEY>)),
    )
    .subcommand(Command::new("rm").about("Remove a KEY").arg(arg!(<KEY>)))
    .get_matches();

  match matches.subcommand() {
    Some(("set", _)) => {
      panic!("unimplemented")
    }
    Some(("get", _)) => {
      panic!("unimplemented")
    }
    Some(("rm", _)) => {
      panic!("unimplemented")
    }
    _ => unimplemented!(),
  }
}
