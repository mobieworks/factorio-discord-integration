use clap::{App, Arg, ArgMatches};

fn main() -> Result<(), ()> {
    let _matches = get_matches();
    Ok(())
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("Factorio Discord Integratoin Tool")
        .version("0.0.1")
        .author("Dopin Ninja <dopinninja@gmail.com>")
        .about("Sends notifications to a Discord channel")
        .arg(
            Arg::with_name("console-log")
                .short("f")
                .help("Sets the console log file to use")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}
