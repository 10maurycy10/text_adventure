use serde;
use std::rc::Rc;
use std::fs;
use serde_json;
use std::io;
use std::collections::HashMap;
use std::io::Write;
use std::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
struct Object {
	desc: Rc<String>,
	can_take: bool,
	names: Vec<String>,
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
struct Place {
	desc: String,
	ambient: Option<String>,
	long: String,
	moves: HashMap<String,String>,
	objects: Vec<Object>
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
struct World {
	map: HashMap<String,Place>,
	location: String,
	aliases: HashMap<String,String>,
	backpack: Vec<Object>,
}

enum NameResolves {
	Mulitple,
	Single(usize),
	Zero,
	Empty_Query,
}

fn get_name(context :Vec<Object>, name :Vec<String>) -> NameResolves {
	let mut acumulator :Vec<bool> = vec![true; context.len()];
	if (name.len() == 0) {
		return NameResolves::Empty_Query;
	}
	for word in name.iter() {	
		for i in 0..context.len() {
			if (!context[i].names.contains(word)) {
				acumulator[i] = false;
			}
		}
	}

	if (acumulator.iter().filter(|&n| *n == true).count() > 1) {
		return NameResolves::Mulitple;
	}
	if (acumulator.iter().filter(|&n| *n == true).count() == 0) {
		return NameResolves::Zero;
	}

	return NameResolves::Single(acumulator.iter().position(|&x| x == true).unwrap());
}

fn command(mut input_str: String, world: &mut World, game_over: &mut bool) {
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
				NameResolves::Empty_Query => println!("You must specify a thing."),
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
				NameResolves::Empty_Query => println!("You must specify a thing."),
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
		println!("{}", new_room.desc);
		println!("{}", new_room.long);
		for i in &new_room.objects {
			println!("you see a {}", i.desc);
		}
	}
}

fn main() {
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
		map : serde_json::from_str(&fs::read_to_string("tree.json").unwrap()).unwrap(),
		location : "_start".to_string(),
		aliases : aliases,
		backpack : Vec::new(),
	};

	let mut game_over = false;
	command("look".to_string(), &mut world, &mut game_over);
	while !game_over {
		print!("> ");
		io::stdout().flush().unwrap();
		let mut input_str = String::new();
		io::stdin().read_line(&mut input_str).unwrap().to_string();
		command(input_str, &mut world, &mut game_over);
	}
}
