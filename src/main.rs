extern crate irc;

use std::time::Instant;
use std::collections::HashMap;

use irc::client::prelude::*;

struct Discourse {
    last_mention: Instant,
    record: Option<u64>,
}

impl Discourse {
    fn new() -> Discourse {
        Discourse {
            last_mention: Instant::now(),
            record: None,
        }
    }

    fn days_since_last(&self) -> u64 {
        self.last_mention.elapsed().as_secs() / (60 * 60 * 24)
    }

    fn seconds_since_last(&self) -> u64 {
        self.last_mention.elapsed().as_secs()
    }

    fn reset(&mut self) {
        let current = Some(self.seconds_since_last());
        self.last_mention = Instant::now();
        self.record = std::cmp::max(self.record, current);
    }
}

fn main() {
    let irc_conf = Config::load("config.json").unwrap();

    println!("Connecting...");

    let srv = IrcServer::from_config(irc_conf).unwrap();
    srv.identify().unwrap();

    println!("Ready!");

    let my_nick = srv.config().nickname.as_ref().unwrap().as_str();

    let mut disco_tracker = HashMap::new();

    for msg in srv.iter() {
        let msg = msg.unwrap();
        match msg.command {
            Command::JOIN(ref channel, _, _) => {
                if let &Some(ref prefix) = &msg.prefix {
                    if prefix.starts_with(my_nick) {
                        println!("Joined to {}.", channel);
                    }
                }
            }
            Command::PRIVMSG(ref target, ref text) => {
                let text = text.trim();
                if let Some(nick) = msg.source_nickname() {
                    let (target, cmd): (&str, Option<&str>) = if target == my_nick {
                        (nick, Some(text))
                    } else {
                        let cmd = if text.starts_with(my_nick) {
                            let text = &text[my_nick.len()..];
                            if text.starts_with(&[',', ':'][..]) {
                                Some(text[1..].trim())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        (target.as_str(), cmd)
                    };

                    if let Some(cmd) = cmd {
                        let disco = disco_tracker.entry(cmd.to_lowercase())
                                                 .or_insert_with(Discourse::new);

                        if let Some(record) = disco.record {
                            disco.reset();
                            srv.send_notice(target,
                                            &format!("It has been [{}] seconds since {} discussed \
                                                     \"{}\".",
                                                      record, target, cmd)).unwrap();
                            srv.send_action(target,
                                            &format!("erases the board, writes [0]")).unwrap();
                        } else {
                            srv.send_notice(target,
                                            &format!("I don't know when the last time {} discussed \
                                                     \"{}\" was, but I'm tracking it now.",
                                                     target, cmd)).unwrap();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
