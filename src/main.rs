extern crate dotenv;
extern crate serenity;

use dotenv::var;

use serenity::Client;
//use serenity::client::Context;
//use serenity::model::Message;
use serenity::model::{ChannelId, User, UserId};
//use serenity::utils::builder::CreateEmbed;
//use serenity::utils::Colour;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime};

#[derive(Clone)]
struct Racer {
	user: User,
	ready: bool,
	finished: Option<SystemTime>,
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

/*struct Race {
	started: Option<SystemTime>,
	players: HashMap<UserId, Racer>,
	ready: u64,
	finished: u64,
}*/

impl Racer {
	fn new(user: User) -> Racer {
		Racer {
			user: user,
			ready: false,
			finished: None,
		}
	}
}

impl Race {
	fn new() -> Race {
		Race::None
	}
}

fn main() {
    let token = var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN Environment Variable");
    let mut client = Client::login_bot(&token);

    let races = Arc::new(Mutex::new(HashMap::<ChannelId, Race>::new()));

	client.with_framework(move |app| {
		let races_open = races.clone();
		let races_enter = races.clone();
		let races_ready = races.clone();
		let races_done = races.clone();
		app
			.configure(|config| config.prefix("!"))
			.on("open", move |_ctx, msg, _v| {
				let chan = msg.channel_id;
				let mut races = races_open.lock().unwrap();
				let race = races.entry(chan).or_insert(Race::None);
				match race {
					Race::None => {
						println!("Opening race in {:?}", chan);
						*race = Race::Open {
							players: HashMap::new(),
							ready: 0,
						};
						chan
							.send_message(|msg| msg.content("race opened: !start to start, !end to end"))
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
				Ok(())
			})
			.on("enter", move |_ctx, msg, _v| {
				let chan = msg.channel_id;
				let mut races = races_enter.lock().unwrap();
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
							players.insert(msg.author.id, Racer::new(msg.author.clone()));
							chan
								.send_message(|smsg| smsg.content(&format!("{:?} entered the race!", msg.author)))
								.unwrap();
						} else {
							chan
								.send_message(|smsg| smsg.content(&format!("{:?} has already entered the race.", msg.author)))
								.unwrap();						
						}
					},
					Race::Running{ .. } => {
						chan
							.send_message(|send| send.content("Can't enter race, it's already in progress!"))
							.unwrap();						
					}
				}
				Ok(())
			})
			.on("ready", move |_ctx, msg, _v| {
				let chan = msg.channel_id;
				let mut races = races_ready.lock().unwrap();
				let race = races.entry(chan).or_insert(Race::None);
				match race {
					Race::None => {
						chan
							.send_message(|send| send.content("Can't enter race, none are open!"))
							.unwrap();
					},
					Race::Open{ ref mut players, ref mut ready } => {
						let player_count = players.len();
						if let Some(ref mut player) = players.get_mut(&msg.author.id) {
							if !player.ready {
								println!("{:?} is ready in {:?}", msg.author, chan);
								player.ready = true;
								*ready += 1;
								chan
									.send_message(|smsg| smsg.content(&format!("{:?} is ready to play!", msg.author)))
									.unwrap();
							} else {
								chan
									.send_message(|smsg| smsg.content(&format!("{:?} was already ready.", msg.author)))
									.unwrap();
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
				Ok(())
			})
			.on("done", move |_ctx, msg, _v| {
				let chan = msg.channel_id;
				let mut races = races_done.lock().unwrap();
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
						if let Some(ref mut player) = players.get_mut(&msg.author.id) {
							if player.finished == None {
								println!("{:?} is finished in {:?}", msg.author, chan);
								player.finished = Some(SystemTime::now());
								*finished += 1;
								chan
									.send_message(|smsg| smsg.content(&format!("{:?} is finished!", msg.author)))
									.unwrap();
							} else {
								chan
									.send_message(|smsg| smsg.content(&format!("{:?} was already finished.", msg.author)))
									.unwrap();
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
				Ok(())				
			})
/*			.on("start", move |_ctx, msg, _v| {
				let mut races = races_start.lock().unwrap();
				let chan = msg.channel_id;
				if let Some(ref mut race) = races.get_mut(&chan) {
					if race.started == None {
						if race.players.len() >= 1 {
							println!("Starting race in {:?}", chan);
							race.started = Some(SystemTime::now());
							chan
								.send_message(|msg| msg.content("race started now! Have fun"))
								.unwrap();
						} else {
							chan
								.send_message(|msg| msg.content("can't start a race with no players :("))
								.unwrap();
						}
					} else {
						chan
							.send_message(|msg| msg.content("race already started :("))
							.unwrap();
					}
				} else {
					chan
						.send_message(|msg| msg.content("no race opened: use !open to open a race"))
						.unwrap();
				}
				Ok(())
			})
			.on("end", move |_ctx, msg, _v| {
				let mut races = races_end.lock().unwrap();
				let chan = msg.channel_id;
				if races.contains_key(&chan) {
					println!("Ending race in {:?}", chan);
					races.remove(&chan);
					chan
						.send_message(|msg| msg.content("race ended"))
						.unwrap();
				} else {
					chan
						.send_message(|msg| msg.content("no race opened, can't end"))
						.unwrap();
				}
				Ok(())
			})*/
	});

    println!("Race bot started");
    client.start().unwrap();
}
