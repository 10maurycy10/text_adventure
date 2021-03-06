use crate::world::*;
use serde_yaml;
use std::fs;
use std::fs::File;
use std::io::Write;

/// ### print command help
fn help() {
    print!(
"\
help : displays available commands; world.aliases = ?
go [north, south, west, east, up, down] : move in a direction; aliases = n s e w u d
take [objects] : take an objects; world.aliases = t [objects]
drop [object] : drop an object form 
look [objects] : look at an object's you ; aliases l
inventory <objects> : look at you backpack ; aliases i
save [file] : save game data to json
load [file] : load game data from json
time : you sit arround ; aliases = .
"   );
}

fn save(input :&mut dyn Iterator<Item = String>, src: &World) {
    match input.next() {
        Some(path) => {
            match File::create(path.clone()) {
                Err(why) => println!("couldn't open {}: {}", path.clone(), why),
                Ok(mut file) => file
                   .write_all(serde_yaml::to_string(src).unwrap().as_bytes())
                   .unwrap(),
                };
            }
        None => println!("You must specify a file path."),
    };
}

fn load(input :&mut dyn Iterator<Item = String>) -> Option<World> {
    match input.next() {
        Some(path) => {
            match fs::read_to_string(path.clone()) {
                Ok(x) => {
                    let deseralized: World = serde_yaml::from_str(&x).unwrap();
                    return Some(deseralized);
                }
                Err(x) => println!("couldn't open {}: {}", path.clone(), x),
            };
        },
        None => println!("please specify the file name"),
    };
    None
}

fn goto(input :&mut dyn Iterator<Item = String>, room :&Place) -> Option<String> {
    match input.next() {
        Some(dir) => match room.moves.get(&dir) {
            Some(dest) => Some(dest.to_string()),
            None => {
                println!("You can't go that way.");
                None
            },
        },
        None => {
            println!("You must specify a direction.");
            None
        },
    }
}

fn take(room: &mut Place, ids : Vec<usize> , player :&mut LazzyCritter) {
    if room.objects[ids[0]].can_take {
        println!("you take the {}", room.objects[ids[0]].f());
        let mut c = player.unpack();
        c.backpack.push(room.objects[ids[0]].clone());
        player.mutate(c);
        room.objects.remove(ids[0]);
    } else {
        println!("nice try but...");
    }
}

fn drop(room: &mut Place, ids : Vec<usize> , player :&mut LazzyCritter) {
    let mut c = player.unpack();
    if c.backpack[ids[0]].can_take {
        println!("you drop the {}", c.backpack[ids[0]].f());
        room.objects.push(c.backpack[ids[0]].clone());
        c.backpack.remove(ids[0]);
    } else {
        println!("nice try but...");
    }
    player.mutate(c);
}

/// attack a target
fn attack(attack :&Attack,target: Critter,room: &mut Place) -> Option<Critter> {
    let mut r = target;
    println!(
        "you {} {}",
        attack.name.p,
        r.desc
    );
    r.hurt(attack.dam);
    if !r.is_dead() {
        println!("{}", r.hurt);
    } else {
        println!(
            "you kill {}",
            r.desc
        );
        r.kill(room);
        return None;
    };
    Some(r)
}

/// # todo REFACTOR THIS!
/// ### pharse command and do it.
pub fn command(mut input_str: String, world: &mut World, game_over: &mut bool) {
    let mut redisplay = false;
    let mut time = true;

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
    let mut curent_room = match world.map.get_mut(&world.player.location) {
        Some(x) => x,
        None => panic!("Room {:?} is not in map", world.player.location),
    };
    let mut input_iter = input.into_iter();
    let start = input_iter.next().unwrap();
    match &*start {
        "help" => help(),
        "save" => save(&mut input_iter, &world),
        "load" => match load(&mut input_iter) {
            Some(x) => *world = x,
            None => ()
        },
        "exit" => *game_over = true,
        "go" => {
            match goto(&mut input_iter,&curent_room) {
                Some(x) => world.player.location = x,
                None => ()
            };
            redisplay = true;
        },
        "look" => {
            redisplay = true;
        },
        "take" => {
            match get_name(
                &curent_room
                    .objects
                    .iter()
                    .map(|x| x.names.clone())
                    .collect(),
                input_iter.collect(),
            ) {
                NameResolves::Results(ids) => take(curent_room,ids,&mut world.player.critter),
                NameResolves::EmptyQuery => println!("You must specify a thing."),
                NameResolves::Zero => println!("You can't find that."),
            }
        }
        "inventory" => {
            println!("HP : {}/{}",world.player.critter.unpack().hp,world.player.critter.unpack().max_hp);
            for i in &world.player.critter.unpack().backpack {
                println!("you have {}", i.desc);
            }
        }
        "drop" => {
            let c = world.player.critter.unpack();
            match get_name(
                &c.backpack.iter().map(|x| x.names.clone()).collect(),
                input_iter.collect(),
            ) {
                NameResolves::Results(ids) => drop(&mut curent_room,ids,&mut world.player.critter),
                NameResolves::EmptyQuery => println!("You must specify a thing."),
                NameResolves::Zero => println!("You can't find that."),
            };
            world.player.critter.mutate(c);
        },
        "eat" => {
            use core::cmp::min;
            let mut c = world.player.critter.unpack();
            match get_name(
                &c.backpack.iter().map(|x| x.names.clone()).collect(),
                input_iter.collect(),
            ) {
                NameResolves::Results(ids) => {
                    if c.backpack[ids[0]].can_take {
                        match c.backpack[ids[0]].food.clone() {
                            Some(x) => {
                                println!("you eat {} for {}", c.backpack[ids[0]].desc, x);
                                c.hp = min(c.max_hp, c.hp + x);
                                c.backpack.remove(ids[0]);
                            },
                            None => println!("you cant eat that")
                        }
                    } else {
                        println!("nice try but...");
                    }
                }
                NameResolves::EmptyQuery => println!("You must specify a thing."),
                NameResolves::Zero => println!("You can't find that."),
            };
            world.player.critter.mutate(c);
        }
        "attack" => {
            let c = world.player.critter.unpack();
            let input_unsplit_vec = input_iter.collect::<Vec<_>>();
            let input = input_unsplit_vec
                .split(|x| *x == "with")
                .map(|xs| xs.iter().collect::<Vec<_>>())
                .collect::<Vec<_>>();
            match input.len() {
                1 => {
                    let _target_id = match get_name(
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
                    ) {
                        NameResolves::EmptyQuery => {
                            println!("JUST TELL ME WHAT TO ATTACK ALREADY!");
                            time = false;
                        },
                        NameResolves::Zero => {
                            println!("You cant see that.");
                            time = false;
                        },
                        NameResolves::Results(ids) => {
                            let target_id = ids[0];
                            let target = curent_room.critters[target_id].unpack();
                            match attack(&c.attack,target.clone(),curent_room) {
                                Some(x) => curent_room.critters[target_id].mutate(x),
                                None => {curent_room.critters.remove(target_id);},
                            };
                        }   
                    };
                },
                2 => {
                    let _target_id = match get_name(
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
                    ) {
                        NameResolves::EmptyQuery => {
                            println!("JUST TELL ME WHAT TO ATTACK ALREADY!");
                            time = false;
                        }
                        NameResolves::Zero => {
                            println!("You cant see that.");
                            time = false;
                        }
                        NameResolves::Results(target_ids) => {
                            let target_id = target_ids[0];
                            let target = curent_room.critters[target_id].unpack();
                            match get_name(
                                &c.backpack.iter().map(|x| x.names.clone()).collect(),
                                (*input[1]
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>())
                                .to_vec(),
                            ) {
                                NameResolves::EmptyQuery => {
                                    println!("JUST TELL ME WHAT TO USE!");
                                    time = false;
                                }
                                NameResolves::Zero => {
                                    println!("You cant find that that.");
                                    time = false;
                                }
                                NameResolves::Results(weppon_ids) => {
                                    let weppon = &c.backpack[weppon_ids[0]];
                                    match weppon.wepon {
                                        Some(ref data) => {
                                            match attack(&data,target.clone(),curent_room) {
                                                Some(x) => curent_room.critters[target_id].mutate(x),
                                                None => {curent_room.critters.remove(target_id);},
                                            }
                                        }
                                        None => {
                                            println!("but...");
                                            time = false;
                                        },
                                    };
                                }
                            };
                        }
                    };
                }
                _ => {
                    println!("Come ON!");
                    time = false;
                },
            }
        },
        "time" => (),
        _ => {
            println!("?");
            time = false;
        },
    }

    let new_room = match world.map.get_mut(&world.player.location) {
        Some(x) => x,
        None => panic!("Room {:?} is not in world.map", world.player.location),
    };

    if time {
        for i in new_room.critters.iter_mut() {
            let mut c = i.unpack();
            c.tick(&mut world.player);
            i.mutate(c)
        }
    }

    if world.player.critter.unpack().hp < 0 {
        println!("you die");
        *game_over = true;
        return;
    }

    if redisplay {
        print_room(new_room);
    }
    if rand::random::<u8>() % 2 == 0 {
        print_amb(new_room);
    }
}
