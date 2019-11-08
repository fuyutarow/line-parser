use std::fs::File;
use std::io::prelude::*;

use chrono;
use regex;
use serde_derive::Serialize;
use structopt::StructOpt;
use toml;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(name = "file")]
    fpath: String,
}

#[derive(Serialize, Debug)]
struct TalkLine {
    title: String,
    at_saved: String,
    messages: Vec<Card>,
}

#[derive(Serialize, Debug)]
struct Card {
    date: String,
    username: String,
    message: String,
}

fn main() {
    let opt = Opt::from_args();
    let filename = opt.fpath;

    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let re_date = regex::Regex::new(
        r"(?x)
(\r\n|\n|\r)(?P<year>\d{4})/(?P<month>\d{2})/(?P<day>\d{2})\((月|火|水|木|金|土|日)\)(\r\n|\n|\r)
",
    )
    .unwrap();

    let dates = re_date.captures_iter(&contents);
    let mut parts = re_date.split(&contents);

    let header = parts.next().unwrap().replace("\u{feff}[LINE] ", "");
    let mut header_lines = header.lines();
    let title = header_lines.next().unwrap().to_string();
    let save_datetime = header_lines.next().unwrap().replace("保存日時：", "");
    let at_saved: String = chrono::prelude::DateTime::parse_from_str(
        &format!("{}:00 +09:00", save_datetime),
        "%Y/%m/%d %H:%M:%S %z",
    )
    .unwrap()
    .to_rfc3339();

    let get_datetime = |date: &regex::Captures, hhmm: &str| {
        chrono::prelude::DateTime::parse_from_str(
            &format!(
                "{}-{}-{} {}:00 +09:00",
                &date["year"], &date["month"], &date["day"], hhmm
            ),
            "%Y-%m-%d %H:%M:%S %z",
        )
        .unwrap()
    };

    let mut card_list: Vec<Card> = Vec::new();
    for (date, part) in dates.zip(parts) {
        for line in part.lines() {
            let v = line.split('\t').collect::<Vec<&str>>();
            match &v[..] {
                [hhmm, username, message] => {
                    let datetime = get_datetime(&date, &hhmm);
                    let card = Card {
                        date: datetime.to_rfc3339(),
                        username: username.to_string(),
                        message: message.to_string(),
                    };
                    card_list.push(card);
                }
                [hhmm, notice] => {
                    let datetime = get_datetime(&date, &hhmm);
                    let card = Card {
                        date: datetime.to_rfc3339(),
                        username: "LINE notification".to_string(),
                        message: notice.to_string(),
                    };
                    card_list.push(card);
                }
                [message] => {
                    if let Some(last) = card_list.last_mut() {
                        last.message.push('\n');
                        last.message.push_str(message);
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    let fname = format!("{}.toml", { &title });
    let talkline = TalkLine {
        title: title,
        at_saved: at_saved,
        messages: card_list,
    };
    let toml = toml::to_string(&talkline).unwrap();
    let mut file = File::create(fname).unwrap();
    writeln!(&mut file, "{}", toml).unwrap();
}
