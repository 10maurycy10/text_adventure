use serde;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct Critter {
	pub hp: u32, //we're not making DF
	pub desc: String,
	pub name: Vec<String>
}

/// wraper for critters
#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub enum LazzyCritter {
	Name(String),
	Critter(Critter)
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct Object {
	pub desc: String,
	pub can_take: bool,
	pub names: Vec<String>,
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct Place {
	desc: String,
	long: String,
	ambient: Option<String>,
	pub moves: HashMap<String,String>,
	pub objects: Vec<Object>,
    pub critters: Vec<LazzyCritter>,
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct World {
	pub map: HashMap<String,Place>,
	pub location: String,
	pub aliases: HashMap<String, String>,
	pub backpack: Vec<Object>,
	/// master copys of all critters
	pub critters: HashMap<String, Critter>,
}

impl LazzyCritter {
	pub fn unpack_init(&mut self, ct: &HashMap<String,Critter>) -> Critter {
		match self {
			LazzyCritter::Name(x) => {
				let t = match ct.get(x) {
					Some(a) => a.clone(),
					None => panic!("unknown critter {:?}", x),
				};
				*self = LazzyCritter::Critter(t.clone());
				self.unpack()
			},
			LazzyCritter::Critter(x) => x.clone(),
		}
	}
	pub fn unpack(&self) -> Critter {
		match self {
			LazzyCritter::Name(_b) => panic!("uninited critter unpacked."),
			LazzyCritter::Critter(x) => x.clone(),
		}
	}
	pub fn mutate(&mut self, new: Critter) {
		match self {
			LazzyCritter::Name(_) => panic!("mutated uninit critter!"),
			LazzyCritter::Critter(x) => *x = new,
		}
	}
}

pub enum NameResolves {
	Mulitple(Vec<usize>),
	Single(usize),
	Zero,
	EmptyQuery,
}

pub fn get_name(context :&Vec<Vec<String>>, name :Vec<String>) -> NameResolves {
	let mut acumulator :Vec<bool> = vec![true; context.len()];
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

	if acumulator.iter().filter(|&n| *n == true).count() > 1 {
		return NameResolves::Mulitple(acumulator.iter().enumerate().filter(|(_, x)| **x == true).map(|(i, _)| i).collect());
	}
	if acumulator.iter().filter(|&n| *n == true).count() == 0 {
		return NameResolves::Zero;
	}

	return NameResolves::Single(acumulator.iter().position(|&x| x == true).unwrap());
}

pub fn print_room(spot :&Place) {
	println!("{}", spot.desc);
	println!("{}", spot.long);
	for i in &spot.objects {
		println!("you see a {}.", i.desc);
	}
	println!("");
	for i in &spot.critters {
		println!("you see a {}.", i.unpack().desc);
	}
}

pub fn print_amb(spot :&Place) {
	match &spot.ambient {
		Some(x) => println!("{}", x),
		None => (),
	}
}	