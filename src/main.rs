use std::fs;
use serde_json;
use std::io;
use std::collections::HashMap;
use std::io::Write;
use std::fs::File;
use clap::{Arg, App};
use std::path::Path;
mod world;
use crate::world::World;

fn command(mut input_str: String, world: &mut World, game_over: &mut bool) {
	 use crate::world::{NameResolves,get_name,print_room,print_amb};
	 let mut redisplay = false;

	if input_str.ends_with('\n') {
		input_str.pop();
	}
	if input_str.ends_with('\r') {
		input_str.pop();
	}
	let mut input : Vec<String> = input_str.split_whitespace().map(|s| s.to_string()).collect();
	if input.len() < 1 {
		println!("?");
		return;
	}

	for i in input.iter_mut() {
		i.make_ascii_lowercase();
	}

	match world.aliases.get(&input[0]) { 
		Some(x) => {
			input.remove(0);
			let alias = x.split_whitespace().map(|s| s.to_string()).collect();
			input = [alias, input].concat();
		},
		None => (),
	};
	let curent_room = match world.map.get_mut(&world.location) {
		Some(x) => x,
		None => panic!("Room {:?} is not in map",world.location),
	};
	let mut input_iter = input.into_iter();
	let start = input_iter.next().unwrap();
	match &*start {
		"help" 	=> {print!(
"\
help : displays available commands; world.aliases = ?
go [north, south, west, east, up, down] : move in a direction; world.aliases = n s e w u d
take [objects] : take an objects; world.aliases = t [objects]
drop [object] : drop an object form 
look [objects] : look at an object's you ; world.aliases l
inventory <objects> : look at you backpack ; world.aliases i
save [file] : save game data to json
load [file] : load game data from json
"	
		);},
		"save" => {
			match input_iter.next() {
				Some(path) => {
					match File::create(path.clone()) {
						Err(why) => println!("couldn't open {}: {}", path.clone(), why),
						Ok(mut file) => file.write_all(serde_json::to_string(world).unwrap().as_bytes()).unwrap(),
					};
				},
				None => println!("You must specify a file path."),
			}
		}
		"load" => match input_iter.next() {
				Some(path) => {
					match fs::read_to_string(path.clone()) {
						Ok(x) => {
							let deseralized: World = serde_json::from_str(&x).unwrap();
							*world = deseralized; 
							redisplay = true;
						},
						Err(x) => println!("couldn't open {}: {}",path.clone(),x),
					};
				},
				None => println!("You must specify a file path."),
		}
		"exit" => {
			*game_over = true;
		}
		"go"	=> {
			match input_iter.next() {
				Some(dir) => match curent_room.moves.get(&dir) {
					Some(dest) => world.location = dest.to_string(),
					None => println!("You can't go that way."),
				},
				None => println!("You must specify a direction."),
			}
			redisplay = true;
		},
		"look" => {
			match input_iter.next() {
				Some(_obj) => unimplemented!(),
				None => (),
			}
			redisplay = true;
		},
		"take" => {
			match get_name(curent_room.objects.clone(), input_iter.collect()) {
				NameResolves::Single(id) => {
					if curent_room.objects[id].can_take {
						println!("you take the {}", curent_room.objects[id].desc);
						world.backpack.push(curent_room.objects[id].clone());
						curent_room.objects.remove(id);
					} else {
						println!("nice try but...");
					}
				}
				NameResolves::EmptyQuery => println!("You must specify a thing."),
				NameResolves::Zero => println!("You can't find that."),
				NameResolves::Mulitple => println!("Be more specific please!"),
			}	
		},
		"inventory" => {
			for i in &world.backpack {
				println!("you have {}", i.desc);
			}
		}
		"drop" => {
			match get_name(world.backpack.clone(), input_iter.collect()) {
				NameResolves::Single(id) => {
					if world.backpack[id].can_take {
						println!("you take the {}", world.backpack[id].desc);
						curent_room.objects.push(world.backpack[id].clone());
						world.backpack.remove(id);
					} else {
						println!("nice try but...");
					}
				}
				NameResolves::EmptyQuery => println!("You must specify a thing."),
				NameResolves::Zero => println!("You can't find that."),
				NameResolves::Mulitple => println!("Be more specific please!"),
			}	
		}
		_ => println!("?"),
	}

	let new_room = match world.map.get_mut(&world.location) {
		Some(x) => x,
		None => panic!("Room {:?} is not in world.map",world.location),
	};
	if redisplay {
		print_room(new_room);
	}
	print_amb(new_room);
}

fn main() {
	let matches = App::new("Text Adventure")
    	.version("0.1.0")
    	.author("Mostly Me")
    	.about("This is an text adventure game enginge")
    	.arg(Arg::with_name("verbose-state")
    	         .short("v")
    	         .long("verbose-state")
    	         .takes_value(false)
    	         .help("Print the state at the end of the turn"))
    	.get_matches();

    let data = Path::new(".").join("data");
	let mut aliases: HashMap<String,String> = HashMap::new();
	aliases.insert("e".to_string(),"go east".to_string());
	aliases.insert("s".to_string(),"go south".to_string());
	aliases.insert("n".to_string(),"go north".to_string());
	aliases.insert("u".to_string(),"go up".to_string());
	aliases.insert("w".to_string(),"go west".to_string());
	aliases.insert("d".to_string(),"go down".to_string());
	aliases.insert("t".to_string(),"take".to_string());
	aliases.insert("q".to_string(),"exit".to_string());
	aliases.insert("?".to_string(),"help".to_string());
	aliases.insert("l".to_string(),"look".to_string());
	aliases.insert("i".to_string(),"inventory".to_string());
	
	let mut world = World {
		map : serde_json::from_str(&fs::read_to_string(data.join("world.json")).unwrap()).unwrap(),
		location : "_start".to_string(),
		aliases : aliases,
		backpack : Vec::new(),
	};

	let mut game_over = false;
	let print_state :bool = matches.value_of("num").is_some();
	command("look".to_string(), &mut world, &mut game_over);
	while !game_over {
		print!("> ");
		io::stdout().flush().unwrap();
		let mut input_str = String::new();
		io::stdin().read_line(&mut input_str).unwrap().to_string();
		command(input_str, &mut world, &mut game_over);
		if print_state {
			println!("{:#?}", world);
		}
	}
}
