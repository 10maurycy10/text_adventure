use crate::world::World;
use serde_json;
use std::fs;
use std::fs::File;
use std::io::Write;

pub fn command(mut input_str: String, world: &mut World, game_over: &mut bool) {
    use crate::world::{get_name, print_amb, print_room, NameResolves};
    let mut redisplay = false;

    if input_str.ends_with('\n') {
        input_str.pop();
    }
    if input_str.ends_with('\r') {
        input_str.pop();
    }
    let mut input: Vec<String> = input_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
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
        }
        None => (),
    };
    let curent_room = match world.map.get_mut(&world.location) {
        Some(x) => x,
        None => panic!("Room {:?} is not in map", world.location),
    };
    let mut input_iter = input.into_iter();
    let start = input_iter.next().unwrap();
    match &*start {
        "help" => {
            print!(
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
            );
        }
        "save" => match input_iter.next() {
            Some(path) => {
                match File::create(path.clone()) {
                    Err(why) => println!("couldn't open {}: {}", path.clone(), why),
                    Ok(mut file) => file
                        .write_all(serde_json::to_string(world).unwrap().as_bytes())
                        .unwrap(),
                };
            }
            None => println!("You must specify a file path."),
        },
        "load" => match input_iter.next() {
            Some(path) => {
                match fs::read_to_string(path.clone()) {
                    Ok(x) => {
                        let deseralized: World = serde_json::from_str(&x).unwrap();
                        *world = deseralized;
                        redisplay = true;
                    }
                    Err(x) => println!("couldn't open {}: {}", path.clone(), x),
                };
            }
            None => println!("You must specify a file path."),
        },
        "exit" => {
            *game_over = true;
        }
        "go" => {
            match input_iter.next() {
                Some(dir) => match curent_room.moves.get(&dir) {
                    Some(dest) => world.location = dest.to_string(),
                    None => println!("You can't go that way."),
                },
                None => println!("You must specify a direction."),
            }
            redisplay = true;
        }
        "look" => {
            match input_iter.next() {
                Some(_obj) => unimplemented!(),
                None => (),
            }
            redisplay = true;
        }
        "take" => {
            match get_name(
                &curent_room
                    .objects
                    .iter()
                    .map(|x| x.names.clone())
                    .collect(),
                input_iter.collect(),
            ) {
                NameResolves::Results(ids) => {
                    if curent_room.objects[ids[0]].can_take {
                        println!("you take the {}", curent_room.objects[ids[0]].f());
                        world.backpack.push(curent_room.objects[ids[0]].clone());
                        curent_room.objects.remove(ids[0]);
                    } else {
                        println!("nice try but...");
                    }
                }
                NameResolves::EmptyQuery => println!("You must specify a thing."),
                NameResolves::Zero => println!("You can't find that."),
            }
        }
        "inventory" => {
            for i in &world.backpack {
                println!("you have {}", i.desc);
            }
        }
        "drop" => {
            match get_name(
                &world.backpack.iter().map(|x| x.names.clone()).collect(),
                input_iter.collect(),
            ) {
                NameResolves::Results(ids) => {
                    if world.backpack[ids[0]].can_take {
                        println!("you drop the {}", world.backpack[ids[0]].f());
                        curent_room.objects.push(world.backpack[ids[0]].clone());
                        world.backpack.remove(ids[0]);
                    } else {
                        println!("nice try but...");
                    }
                }
                NameResolves::EmptyQuery => println!("You must specify a thing."),
                NameResolves::Zero => println!("You can't find that."),
            }
        }
        "attack" => {
            let input_unsplit_vec = input_iter.collect::<Vec<_>>();
            let input = input_unsplit_vec
                .split(|x| *x == "with")
                .map(|xs| xs.iter().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            match input.len() {
                1 => println!("What?"),
                2 => {
                    let target_id = match (get_name(
                        &curent_room
                            .critters
                            .iter()
                            .map(|x| x.unpack().name.clone())
                            .collect(),
                        (*input[0]
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>())
                        .to_vec(),
                    )) {
                        NameResolves::EmptyQuery => {
                            println!("JUST TELL ME WHAT TO ATTACK ALREADY!");
                        }
                        NameResolves::Zero => {
                            println!("You cant see that.");
                        }
                        NameResolves::Results(target_ids) => {
                            let target_id = target_ids[0];
                            let target = curent_room.critters[target_id].unpack();
                            match get_name(
                                &world.backpack.iter().map(|x| x.names.clone()).collect(),
                                (*input[1]
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>())
                                .to_vec(),
                            ) {
                                NameResolves::EmptyQuery => {
                                    println!("JUST TELL ME WHAT TO USE!");
                                }
                                NameResolves::Zero => {
                                    println!("You cant find that that.");
                                }
                                NameResolves::Results(weppon_ids) => {
                                    let weppon = &world.backpack[weppon_ids[0]];
                                    match weppon.wepon {
                                        Some(ref data) => {
                                            println!(
                                                "you attack {} with {}",
                                                target.desc,
                                                weppon.f()
                                            );
                                            let new_hp = target.hp - data.dam;
                                            if (new_hp > 0) {
                                                let mut n_c = target.clone();
                                                n_c.hp = new_hp;
                                                curent_room.critters[target_id].mutate(n_c);
                                                println!("{}", target.hurt);
                                            } else {
                                                println!(
                                                    "you kill {} with {}",
                                                    target.desc,
                                                    weppon.f()
                                                );
                                                curent_room.critters.remove(target_id);
                                            }
                                        }
                                        None => println!("but..."),
                                    };
                                }
                            }
                        }
                    };
                }
                _ => println!("Come ON!"),
            }
        }
        _ => println!("?"),
    }

    let new_room = match world.map.get_mut(&world.location) {
        Some(x) => x,
        None => panic!("Room {:?} is not in world.map", world.location),
    };
    if redisplay {
        print_room(new_room);
    }
    if rand::random::<u8>() % 2 == 0 {
        print_amb(new_room);
    }
}
