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

struct Racer {
	user: User,
	ready: bool,
	finished: Option<SystemTime>,
}

struct Race {
	started: Option<SystemTime>,
	players: HashMap<UserId, Racer>,
	ready: u64,
	finished: u64,
}

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
		Race {
			started: None,
			players: HashMap::new(),
			ready: 0,
			finished: 0,
		}
	}
}

fn main() {
    let token = var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN Environment Variable");
    let mut client = Client::login_bot(&token);

    let races = Arc::new(Mutex::new(HashMap::<ChannelId, Race>::new()));

	client.with_framework(move |app| {
		let races_open = races.clone();
		let races_enter = races.clone();
		let races_start = races.clone();
		let races_end = races.clone();
		app
			.configure(|config| config.prefix("!"))
			.on("open", move |_ctx, msg, _v| {
				let mut races = races_open.lock().unwrap();
				let chan = msg.channel_id;
				if !races.contains_key(&chan) {
					println!("Opening race in {:?}", chan);
					races.insert(chan.clone(), Race::new());
					chan
						.send_message(|msg| msg.content("race opened: !start to start, !end to end"))
						.unwrap();
				} else {
					chan
						.send_message(|msg| msg.content("race already opened"))
						.unwrap();
				}
				Ok(())
			})
			.on("enter", move |_ctx, msg, _v| {
				let mut races = races_enter.lock().unwrap();
				let chan = msg.channel_id;
				if let Some(ref mut race) = races.get_mut(&chan) {
					if race.started != None {
						chan
							.send_message(|smsg| smsg.content("Can't enter race, it's already started"))
							.unwrap();						
					} else if race.players.contains_key(&msg.author.id) {
						chan
							.send_message(|smsg| smsg.content(&format!("{:?} has already entered the race.", msg.author)))
							.unwrap();
					} else {
						println!("{:?} entered race in {:?}", msg.author, chan);
						race.players.insert(msg.author.id, Racer::new(msg.author.clone()));
						chan
							.send_message(|smsg| smsg.content(&format!("{:?} entered the race!", msg.author)))
							.unwrap();
					}
				} else {
					chan
						.send_message(|msg| msg.content("No race open!"))
						.unwrap();
				}
				Ok(())
			})
			.on("start", move |_ctx, msg, _v| {
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
			})
	});

    println!("Race bot started");
    client.start().unwrap();
}
