use clap::{App, Arg, ArgMatches};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use reqwest::Client;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

#[tokio::main]
#[allow(unused_must_use)]
async fn main() -> Result<(), ()> {
    let matches = get_matches();
    let path = matches.value_of("path").expect("bad console log file path");
    let url = matches.value_of("url").expect("bad webhook url");
    let client = reqwest::Client::new();

    run(&path, &client, &url).await;

    Ok(())
}

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("Factorio Discord Integratoin Tool")
        .version("0.0.1")
        .author("Dopin Ninja")
        .about("Sends notifications to a Discord channel")
        .arg(
            Arg::with_name("path")
                .short("f")
                .help("Console log file path")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("webhook")
                .help("Discord webhook URL")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}

async fn run(path: &str, client: &reqwest::Client, url: &str) -> notify::Result<()> {
    let file = match OpenOptions::new().read(true).open(path) {
        Err(why) => panic!("Cannot open file! file:{} cause:{}", path, why.to_string()),
        Ok(file) => file,
    };
    let mut reader = BufReader::new(&file);
    let metadata = match file.metadata() {
        Err(why) => panic!("Cannot read file metadata :{}", why.to_string()),
        Ok(data) => data,
    };
    let file_size = metadata.len();
    tail_file_follow(&mut reader, path, file_size, client, url).await?;

    Ok(())
}

#[allow(unused_must_use)]
async fn tail_file_follow(
    reader: &mut BufReader<&File>,
    spath: &str,
    file_size: u64,
    client: &reqwest::Client,
    url: &str,
) -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;
    let path = Path::new(spath);
    watcher.watch(path, RecursiveMode::NonRecursive)?;
    let mut start_byte = file_size;
    let mut buf_str = String::new();
    loop {
        match rx.recv() {
            Err(e) => println!("watch error: {:?}", e),
            Ok(_) => {
                match reader.seek(SeekFrom::Start(start_byte)) {
                    Err(why) => panic!(
                        "Cannot move offset! offset:{} cause:{}",
                        start_byte,
                        why.to_string()
                    ),
                    Ok(_) => start_byte,
                };
                let read_byte = match reader.read_to_string(&mut buf_str) {
                    Err(why) => panic!(
                        "Cannot read offset byte! offset:{} cause:{}",
                        start_byte,
                        why.to_string()
                    ),
                    Ok(b) => b,
                };
                start_byte += read_byte as u64;
                post(&buf_str, client, url).await;
                buf_str.clear();
            }
        }
    }
}

async fn post(text: &str, client: &reqwest::Client, url: &str) -> Result<(), reqwest::Error> {
    for line in text.lines() {
        let mut strings = line.trim_start().split_whitespace();
        if strings.next() == None {
            // Empty line
            continue;
        }
        let tag = strings.nth(1).unwrap_or("Unknown tag");
        if tag == "[JOIN]" {
            http_post(
                &format!(
                    "{}が出勤しました。",
                    strings.next().unwrap_or("<Username N/A>")
                ),
                client,
                url,
            )
            .await?;
        } else if tag == "[LEAVE]" {
            http_post(
                &format!(
                    "{}が退勤しました。",
                    strings.next().unwrap_or("<Username N/A>")
                ),
                client,
                url,
            )
            .await?;
        }
    }

    Ok(())
}

async fn http_post(text: &str, client: &Client, url: &str) -> Result<(), reqwest::Error> {
    client
        .post(url)
        .json(&serde_json::json!({ "content": text }))
        .send()
        .await?;
    Ok(())
}
