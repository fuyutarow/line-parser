use std::env;
use std::fs::File;
use std::io::prelude::*;

use chrono;
use serde_derive::Serialize;
use toml;

#[derive(Serialize, Debug)]
struct TalkLine {
    title: String,
    messages: Vec<Card>,
}

#[derive(Serialize, Debug)]
struct Card {
    date: String,
    username: String,
    message: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    let mut lines = contents.lines();

    let title = lines.next().unwrap().replace("\u{feff}[LINE] ", "");

    let mut date = "";
    let mut CardList: Vec<Card> = Vec::new();
    for line in lines {
        if let Some(first4) = &line.split('/').next() {
            if vec!["2014", "2015", "2016", "2017", "2018", "2019"].contains(first4) {
                date = line.split('(').next().unwrap();
                if let Some(last) = CardList.last_mut() {
                    last.message.pop();
                }
                continue;
            }
        }

        let v = &line.split('\t').collect::<Vec<&str>>();
        match &v[..] {
            [HHmm, username, message] => {
                let datetime = chrono::prelude::DateTime::parse_from_str(
                    &format!("{} {}:00 +09:00", date, HHmm),
                    "%Y/%m/%d %H:%M:%S %z",
                )
                .unwrap();
                let card = Card {
                    date: datetime.to_rfc3339(),
                    username: username.to_string(),
                    message: message.to_string(),
                };
                CardList.push(card);
            }
            [HHmm, notice] => {
                let datetime = chrono::prelude::DateTime::parse_from_str(
                    &format!("{} {}:00 +09:00", date, HHmm),
                    "%Y/%m/%d %H:%M:%S %z",
                )
                .unwrap();
                let card = Card {
                    date: datetime.to_rfc3339(),
                    username: "LINE notification".to_string(),
                    message: notice.to_string(),
                };
                CardList.push(card);
            }
            [message] => {
                if let Some(last) = CardList.last_mut() {
                    last.message.push('\n');
                    last.message.push_str(message);
                }
            }
            _ => unreachable!(),
        }
    }

    let fname = format!("{}.toml", { &title });
    let talkline = TalkLine {
        title: title,
        messages: CardList,
    };
    let toml = toml::to_string(&talkline).unwrap();
    let mut file = File::create(fname).unwrap();
    writeln!(&mut file, "{}", toml).unwrap();
}
