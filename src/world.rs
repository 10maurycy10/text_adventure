use rand;
use rand::Rng;
use serde;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ### a colection of names to avoid gramer errors
/// punch punches punched 
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Action {
    pub p: String,
    pub s: String,
    pub pt: String
}


/// ### the default behavior of critters
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Alignment {
	/// chill unless attacked
    Fine,
    /// always mad
	Evil,
}

/// ### the curent mood
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Anoyance {
    ///sits around
	Chill,
    ///attacks
	Mad,
}

impl Default for Anoyance {
	fn default() -> Self { Anoyance::Chill }
}

/// ### natural atack
/// The attack for a critter that dose not have items
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Attack {
	pub name: Action,
	pub dam: i32
}

/// ### a contaner for critter data
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Critter {
    /// ### the natural attack
	pub attack: Attack, 
    /// ### the hit-points (THIS IS NOT DF)
    pub hp: i32,
    pub max_hp: i32,
    /// ### the discrition
    pub desc: String,
    /// ### the names
    pub name: Vec<String>,
    /// ### the noise made by the critter
    pub noise: Option<String>,
    /// ### optional
	#[serde(default)]
    pub anoyance: Anoyance,
    pub alignment: Alignment,
    /// ### the hurt noise
    pub hurt: String,
    /// ### optional
    /// not used by npc (for now)
    #[serde(default)]
    pub backpack: Vec<Object>,
}

impl Default for Critter {
    fn default() -> Self {
        Critter {
            attack : Attack {
                name : Action {
                    s : "".to_string(),
                    p : "".to_string(),
                    pt : "".to_string(),
                },
                dam : 0
            },
            hp : 0,
            name : vec!("".to_string()),
            max_hp : 1,
            desc: "".to_string(),
            noise: None,
            anoyance: Anoyance::Chill,
            alignment: Alignment::Fine,
            hurt : "".to_string(),
            backpack: Vec::new()
        }
    }
}

/// ### wraper for critters
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum LazzyCritter {
    /// look up address
    Name(String),
    /// the raw data
    Critter(Critter),
}

/// ### an Item
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Object {
    pub desc: String,
    pub can_take: bool,
    pub names: Vec<String>,
    pub wepon: Option<Attack>,
    pub food: Option<i32>,
}

impl Object {
    /// ### generate a name
    pub fn f(&self) -> String {
        format!(
            "{} {}",
            self.desc,
            if self.wepon.is_some() { "[WPN]" } else { "" }
        )
    }
}


/// ### a location
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Place {
    desc: String,
    long: String,
    ambient: Option<String>,
    pub moves: HashMap<String, String>,
    pub objects: Vec<Object>,
    pub critters: Vec<LazzyCritter>,
}

/// ### the data for player data
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Player {
    /// the critter used for all other data
	pub critter: LazzyCritter,
	pub location: String,
}

/// ### a contaner for game data
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct World {
    pub map: HashMap<String, Place>,
    pub aliases: HashMap<String, String>,
 	pub player: Player,	
    pub critters: HashMap<String, Critter>,
}

impl LazzyCritter {
    /// unpack w/ lookup
    pub fn unpack_init(&mut self, ct: &HashMap<String, Critter>) -> Critter {
        match self {
            LazzyCritter::Name(x) => {
                let t = match ct.get(x) {
                    Some(a) => a.clone(),
                    None => panic!("unknown critter {:?}", x),
                };
                *self = LazzyCritter::Critter(t.clone());
                self.unpack()
            }
            LazzyCritter::Critter(x) => x.clone(),
        }
    }
    pub fn unpack(&self) -> Critter {
        match self {
            LazzyCritter::Name(_b) => panic!("uninited critter unpacked."),
            LazzyCritter::Critter(x) => x.clone(),
        }
    }
    /// change data
    pub fn mutate(&mut self, new: Critter) {
        match self {
            LazzyCritter::Name(_) => panic!("mutated uninit critter!"),
            LazzyCritter::Critter(x) => *x = new,
        }
    }
}

#[derive(Debug,Eq,PartialEq)]
pub enum NameResolves {
    Results(Vec<usize>),
    Zero,
    EmptyQuery,
}

/// ### name resolving
/// all name entry must match
pub fn get_name(context: &Vec<Vec<String>>, name: Vec<String>) -> NameResolves {
    let mut acumulator: Vec<bool> = vec![true; context.len()];
    if name.len() == 0 {
        return NameResolves::EmptyQuery;
    }
    for word in name.iter() {
        for i in 0..context.len() {
            if !context[i].contains(word) {
                acumulator[i] = false;
            }
        }
    }

    if acumulator.iter().filter(|&n| *n == true).count() > 0 {
        return NameResolves::Results(
            acumulator
                .iter()
                .enumerate()
                .filter(|(_, x)| **x == true)
                .map(|(i, _)| i)
                .collect(),
        );
    } else {
        return NameResolves::Zero;
    }
}

impl Critter {
	pub fn tick(&mut self, player :&mut Player) {
		match self.anoyance {
			Anoyance::Chill => (),   
			Anoyance::Mad => {
				let attack = &self.attack;
				println!("{} {} you.", self.desc, attack.name.s);
				let mut c = player.critter.unpack();
                c.hp -= attack.dam;
                player.critter.mutate(c);
			} 
		}
	}
	pub fn hurt(&mut self, dam : i32) {
		self.anoyance = Anoyance::Mad;
		//println!("hurting {} for {}", self.desc, dam);
        self.hp -= dam;
	}
	pub fn is_dead(&self) -> bool {
		self.hp < 0
	}
    /// call this when killing  
    pub fn kill(&mut self, p: &mut Place) {
        p.objects.append(&mut self.backpack);
    }
}

pub fn print_room(spot: &Place) {
    println!("{}", spot.desc);
    println!("{}", spot.long);
    for i in &spot.objects {
        println!("you see a {}.", i.f());
    }
    println!("");
    for i in &spot.critters {
        println!("you see a {}.", i.unpack().desc);
    }
}

pub fn print_amb(spot: &Place) {
    match &spot.ambient {
        Some(x) => println!("{}", x),
        None => (),
    }
    for i in &spot.critters {
        match i.unpack().noise {
            Some(a) => {
                if rand::thread_rng().gen_range(0, 5) == 0 {
                    println!("* {}",a);
                }
            }
            None => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_name_resolving() {
        let t = vec!(
            vec!("a".to_string(),"b".to_string()),
            vec!("a".to_string(),"c".to_string())
        );
        assert_eq!(get_name(&t,vec!("a".to_string())), NameResolves::Results(vec!(0,1)));
        assert_eq!(get_name(&t,vec!("b".to_string())), NameResolves::Results(vec!(0)));
        assert_eq!(get_name(&t,vec!("c".to_string())), NameResolves::Results(vec!(1)));
        assert_eq!(get_name(&t,vec!("a".to_string(),"b".to_string())), NameResolves::Results(vec!(0)));
        assert_eq!(get_name(&t,vec!("d".to_string())), NameResolves::Zero);
        assert_eq!(get_name(&t,vec!()), NameResolves::EmptyQuery);
    }
    #[test]
    fn test_lazzy_critter_init() {
        let mut c = LazzyCritter::Name("my-c".to_string());
        let mut map: HashMap<String,Critter> = HashMap::new();
        map.insert("my-c".to_string(), Critter::default());
        c.unpack_init(&map);
        assert_eq!(c,  LazzyCritter::Critter(Critter::default()));
    }
    #[should_panic]
    #[test]
    fn test_lazzy_critter_uninit_panic() {
        let mut c = LazzyCritter::Name("my-c".to_string());
        let map: HashMap<String,Critter> = HashMap::new();
        c.unpack_init(&map);
    }
    #[should_panic]
    #[test]
    fn test_lazzy_critter_uninit_panic2() {
        let mut c = LazzyCritter::Name("my-c".to_string());
        c.mutate(Critter::default)
    }
    #[test]
    fn test_lazzy_critter_mutate() {
        let mut c = LazzyCritter::Critter(Critter::default());
        c.mutate(Critter::default());
        let mut nc = Critter::default();
        nc.hp =- 1;
        c.mutate(nc);
        assert!(c.unpack() != Critter::default());

    }
    #[test]
    fn test_critter_dead() {
        let mut c = Critter::default();
        c.hp = 1;
        assert_eq!(c.is_dead(), false);
        c.hurt(1);
        assert_eq!(c.is_dead(), false);
        c.hurt(1);
        assert_eq!(c.is_dead(), true);
    }
}