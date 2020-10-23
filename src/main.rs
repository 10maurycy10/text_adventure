use clap::{App, Arg};
use serde_yaml;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
mod command;
mod world;
use crate::command::command;
use crate::world::*;

fn main() {
    let matches = App::new("Text Adventure")
        .version("0.1.0")
        .author("Mostly Me")
        .about("This is an text adventure game enginge")
        .arg(
            Arg::with_name("verbose-state")
                .short("v")
                .long("verbose-state")
                .takes_value(false)
                .help("Print the state at the end of the turn"),
        )
        .get_matches();

    let data = Path::new(".").join("data");
    let mut aliases: HashMap<String, String> = HashMap::new();
    aliases.insert("e".to_string(), "go east".to_string());
    aliases.insert("s".to_string(), "go south".to_string());
    aliases.insert("n".to_string(), "go north".to_string());
    aliases.insert("u".to_string(), "go up".to_string());
    aliases.insert("w".to_string(), "go west".to_string());
    aliases.insert("d".to_string(), "go down".to_string());
    aliases.insert("t".to_string(), "take".to_string());
    aliases.insert("q".to_string(), "exit".to_string());
    aliases.insert("?".to_string(), "help".to_string());
    aliases.insert("l".to_string(), "look".to_string());
    aliases.insert("i".to_string(), "inventory".to_string());

    let mut world = World {
        map: serde_yaml::from_str(&fs::read_to_string(data.join("world.yml")).unwrap()).unwrap(),
        critters: serde_yaml::from_str(&fs::read_to_string(data.join("critters.yml")).unwrap())
            .unwrap(),
        player: serde_yaml::from_str(&fs::read_to_string(data.join("player.yml")).unwrap())
            .unwrap(),
        aliases: aliases,
        backpack: Vec::new(),
    };

    for room in world.map.iter_mut() {
        for critter in room.1.critters.iter_mut() {
            critter.unpack_init(&(world.critters)); //force all to be inited
            match critter.unpack().alignment { //init anger
                Alignment::Fine => (),
                Alignment::Evil => {
                    let mut new = critter.unpack();
                    new.anoyance = Anoyance::Mad;
                    critter.mutate(new);
                },
            }
        }
    }

    let mut game_over = false;
    let print_state: bool = matches.value_of("num").is_some();
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
