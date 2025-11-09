use std::env::current_dir;
use std::process::exit;

use clap::{arg, command, Command};
use kvs::Error;
use kvs::Result;
use kvs::DB;

fn main() -> Result<()> {
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
        Some(("set", matches)) => {
            let key = matches.get_one::<String>("key").expect("required");
            let value = matches.get_one::<String>("value").expect("required");

            let mut db = DB::open(current_dir()?)?;
            db.set(key.to_string(), value.to_string())?;
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("key").expect("required");

            let mut db = DB::open(current_dir()?)?;
            if let Some(value) = db.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Some(("rm", matches)) => {
            let key = matches.get_one::<String>("key").expect("required");

            let mut db = DB::open(current_dir()?)?;
            match db.remove(key.to_string()) {
                Ok(()) => {}
                Err(Error::KeyNotFound) => {
                    println!("Key not found");
                    exit(1)
                }
                Err(e) => return Err(e),
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
