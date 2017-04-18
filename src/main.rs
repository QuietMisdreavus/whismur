// whismur (discourse-bot) - a bot to track duration between certain topics
// Copyright (C) 2017 QuietMisdreavus
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate irc;

use std::time::Instant;
use std::collections::HashMap;

use irc::client::prelude::*;

#[derive(Debug)]
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

    fn reset(&mut self) {
        let current = Some(self.days_since_last());
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
                        let tracker = disco_tracker.entry(target.to_owned())
                                                   .or_insert_with(HashMap::new);
                        let disco = tracker.entry(cmd.to_lowercase())
                                           .or_insert_with(Discourse::new);

                        if let Some(record) = disco.record {
                            let current = disco.days_since_last();
                            srv.send_notice(target,
                                            &format!("It has been [{}] days since {} discussed \
                                                     \"{}\".",
                                                      current, target, cmd)).unwrap();
                            srv.send_action(target,
                                            &format!("erases the board, writes 0")).unwrap();
                            srv.send_notice(target,
                                            &format!("The previous record was [{}] days.",
                                                     record)).unwrap();
                            disco.reset();
                        } else {
                            disco.record = Some(0);
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
