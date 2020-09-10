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
struct Item {
	desc: Rc<String>,
	can_take: bool,
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
struct Place {
	desc: String,
	ambient: Option<String>,
	long: String,
	moves: HashMap<String,String>,
	objects: HashMap<String, Item>
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
struct World {
	map: HashMap<String,Place>,
	location: String,
	aliases: HashMap<String,String>,
	backpack: HashMap<String,Item>,
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
	let mut input_iter = input.iter();
	let start = input_iter.next().unwrap();
	match &**start {
		"help" 	=> {print!(
"\
help : dispalys avalable comands; world.aliases = ?
go [north, south, west, east, up, down] : move in a direction; world.aliases = n s e w u d
take [objects] : take an objects; world.aliases = t [objects]
look [objects] : look at an objects you ; world.aliases l
inventory <objects> : look at you backpack ; world.aliases i
"	
		);},
		"save" => {
			match input_iter.next() {
				Some(path) => {
					match File::create(path) {
						Err(why) => println!("couldn't open {}: {}", path, why),
						Ok(mut file) => file.write_all(serde_json::to_string(world).unwrap().as_bytes()).unwrap(),
					};
				},
				None => println!("You must specify a file path."),
			}
		}
		"load" => match input_iter.next() {
				Some(path) => {
					match fs::read_to_string(path) {
						Ok(x) => {
							let deseralized: World = serde_json::from_str(&x).unwrap();
							*world = deseralized; 
							redisplay = true;
						},
						Err(x) => println!("couldn't open {}: {}",path,x),
					};
				},
				None => println!("You must specify a file path."),
		}
		"exit" => {
			*game_over = true;
		}
		"go"	=> {
			match input_iter.next() {
				Some(dir) => match curent_room.moves.get(dir) {
					Some(dest) => world.location = dest.to_string(),
					None => println!("You can not go that way."),
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
			match input_iter.next() {
				Some(thing) => {
					match curent_room.objects.get(thing) {
						Some(that) => {
							if that.can_take {
								println!("you take the {}", that.desc);
								{world.backpack.insert(thing.to_string(),that.clone());}
							} else {
								println!("nice try but...");
							}
						},
						None => println!("You can find that."),
					};
					if curent_room.objects.contains_key(thing) {
						if curent_room.objects.get(thing).unwrap().can_take {
							curent_room.objects.remove(thing);
						}
					}
				}
				None => println!("You must specify a thing."),
			}	
		},
		"inventory" => {
			for i in &world.backpack {
				println!("you have {}", i.1.desc);
			}
		}
		"drop" => {
			match input_iter.next() {
				Some(thing) => {
					match world.backpack.get(thing) {
						Some(that) => {
							if that.can_take {
								println!("you drop the {}", that.desc);
								curent_room.objects.insert(thing.to_string(),that.clone());
							} else {
								println!("nice try but...");
							}
						},
						None => println!("You can find that."),
					};
					world.backpack.remove(thing);
				}
				None => println!("You must specify a thing."),
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
			println!("you see a {}", i.1.desc);
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
		backpack : HashMap::new(),
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
