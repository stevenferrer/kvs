use clap::{arg, command, Command};

fn main() {
    let cmd = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("get")
                .about("get value for a given [key]")
                .arg(arg!([key]).required(true)),
        )
        .subcommand(
            Command::new("set")
                .about("set [value] for a given [key]")
                .arg(arg!([key]).required(true))
                .arg(arg!([value]).required(true)),
        )
        .subcommand(
            Command::new("rm")
                .about("remove a given [key]")
                .arg(arg!([key]).required(true)),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("get", _)) => todo!("unimplemented"),
        Some(("set", _)) => todo!("unimplemented"),
        Some(("rm", _)) => todo!("unimplemented"),
        _ => unreachable!(),
    }
}
