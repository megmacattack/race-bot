extern crate dotenv;
extern crate ordinal;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serenity;

use dotenv::var;

use ordinal::Ordinal;

use serenity::Client;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::user::User;
use serenity::prelude::EventHandler;
use serenity::framework::standard::StandardFramework;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime};

#[derive(Clone)]
enum Racer {
	Entered { user: User },
	Ready { user: User },
	Finished { user: User, time: SystemTime },
}

enum Race {
	None,
	Open {
		players: HashMap<UserId, Racer>,
		ready: usize,
	},
	Running {
		started: SystemTime,
		players: HashMap<UserId, Racer>,
		finished: usize,
	},
}

lazy_static! {
	static ref RACES: Arc<Mutex<HashMap<ChannelId, Race>>> = { Arc::new(Mutex::new(HashMap::new())) };
}

command!(open_race(_ctx, msg) {
	let chan = msg.channel_id;
	let mut races = RACES.lock().unwrap();
	let race = races.entry(chan).or_insert(Race::None);
	match race {
		Race::None => {
			println!("Opening race in {:?}", chan);
			*race = Race::Open {
				players: HashMap::new(),
				ready: 0,
			};
			chan
				.send_message(|msg| msg.content("race opened: Please !enter the race and !ready once ready"))
				.unwrap();

		},
		Race::Open{..} => {
			chan
				.send_message(|msg| msg.content("already opened!"))
				.unwrap();						
		},
		Race::Running{..} => {
			chan
				.send_message(|msg| msg.content("race already in progress!"))
				.unwrap();
		}
	}
});

command!(enter_race(_ctx, msg) {
	let chan = msg.channel_id;
	let mut races = RACES.lock().unwrap();
	let race = races.entry(chan).or_insert(Race::None);
	match race {
		Race::None => {
			chan
				.send_message(|send| send.content("Can't enter race, none are open!"))
				.unwrap();
		},
		Race::Open{ ref mut players, .. } => {
			if !players.contains_key(&msg.author.id) {
				println!("{:?} entered race in {:?}", msg.author, chan);
				players.insert(msg.author.id, Racer::Entered { user: msg.author.clone() });
				chan
					.send_message(|smsg| smsg.content(&format!("{} entered the race!", msg.author)))
					.unwrap();
			} else {
				chan
					.send_message(|smsg| smsg.content(&format!("{} has already entered the race.", msg.author)))
					.unwrap();						
			}
		},
		Race::Running{ .. } => {
			chan
				.send_message(|send| send.content("Can't enter race, it's already in progress!"))
				.unwrap();						
		}
	}

});

command!(ready_for_race(_ctx, msg) {
	let chan = msg.channel_id;
	let mut races = RACES.lock().unwrap();
	let race = races.entry(chan).or_insert(Race::None);
	match race {
		Race::None => {
			chan
				.send_message(|send| send.content("Can't enter race, none are open!"))
				.unwrap();
		},
		Race::Open{ ref mut players, ref mut ready } => {
			let player_count = players.len();

			if let Some(player) = players.get_mut(&msg.author.id) {
				match player {
					Racer::Entered { user } => {
						println!("{:?} is ready in {:?}", msg.author, chan);
						*player = Racer::Ready { user: user.clone() };
						*ready += 1;
						chan
							.send_message(|smsg| smsg.content(&format!("{} is ready to play!", msg.author)))
							.unwrap();
					},
					Racer::Ready { .. } => {
						chan
							.send_message(|smsg| smsg.content(&format!("{} was already ready.", msg.author)))
							.unwrap();
					},
					Racer::Finished { .. } => { panic!("Finished racer when race not running??") },
				}
			}

			if *ready == player_count {
				println!("all players are ready, starting race in {:?}", chan);
				chan
					.send_message(|smsg| smsg.content("All players ready, starting race!"))
					.unwrap();

				*race = Race::Running {
					started: SystemTime::now(),
					players: players.clone(),
					finished: 0
				};
			}
		},
		Race::Running{ .. } => {
			chan
				.send_message(|send| send.content("Can't ready up, game in progress."))
				.unwrap();						
		}
	}

});

command!(done_race(_ctx, msg) {
	let chan = msg.channel_id;
	let mut races = RACES.lock().unwrap();
	let race = races.entry(chan).or_insert(Race::None);
	match race {
		Race::None => {
			chan
				.send_message(|send| send.content("Can't finish race, it's not open yet!"))
				.unwrap();
		},
		Race::Open{ .. } => {
			chan
				.send_message(|send| send.content("Can't finish race, it's not started yet!"))
				.unwrap();
		},
		Race::Running{ started: _, ref mut players, ref mut finished } => {
			let player_count = players.len();
			if let Some(player) = players.get_mut(&msg.author.id) {
				match player {
					Racer::Entered {..} => { panic!("Entered racer in running race??"); },
					Racer::Ready { user } => {
						println!("{:?} is finished in {:?}", msg.author, chan);
						*player = Racer::Finished {
							user: user.clone(),
							time: SystemTime::now(),
						};
						*finished += 1;
						chan
							.send_message(|smsg| smsg.content(&format!("{} finished in {} place!", msg.author, Ordinal::from(*finished))))
							.unwrap();
					},
					Racer::Finished {..} => {
						chan
							.send_message(|smsg| smsg.content(&format!("{} was already finished.", msg.author)))
							.unwrap();
					}
				}
			}

			if *finished == player_count {
				println!("all players are finished, ending race in {:?}", chan);
				chan
					.send_message(|smsg| smsg.content("All players finished, race is over!"))
					.unwrap();

				*race = Race::None;
			}
		}
	}

});

struct Handler;

impl EventHandler for Handler {}

fn main() {
    let token = var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN Environment Variable");
    let mut client = Client::new(&token, Handler).expect("Failure creating discord client");

	client.with_framework(StandardFramework::new()
		.configure(|config| config.prefix("!"))
		.cmd("open", open_race)
		.cmd("enter", enter_race)
		.cmd("ready", ready_for_race)
		.cmd("done", done_race)
	);

    println!("Race bot started");
    client.start().unwrap();
}
