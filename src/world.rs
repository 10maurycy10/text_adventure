use serde;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};


#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct Object {
	pub desc: String,
	pub can_take: bool,
	names: Vec<String>,
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct Place {
	desc: String,
	long: String,
	ambient: Option<String>,
	pub moves: HashMap<String,String>,
	pub objects: Vec<Object>
}

#[derive(Debug,Clone,Deserialize,Serialize,PartialEq,Eq)]
pub struct World {
	pub map: HashMap<String,Place>,
	pub location: String,
	pub aliases: HashMap<String,String>,
	pub backpack: Vec<Object>,
}

pub enum NameResolves {
	Mulitple,
	Single(usize),
	Zero,
	EmptyQuery,
}

pub fn get_name(context :Vec<Object>, name :Vec<String>) -> NameResolves {
	let mut acumulator :Vec<bool> = vec![true; context.len()];
	if name.len() == 0 {
		return NameResolves::EmptyQuery;
	}
	for word in name.iter() {	
		for i in 0..context.len() {
			if !context[i].names.contains(word) {
				acumulator[i] = false;
			}
		}
	}

	if acumulator.iter().filter(|&n| *n == true).count() > 1 {
		return NameResolves::Mulitple;
	}
	if acumulator.iter().filter(|&n| *n == true).count() == 0 {
		return NameResolves::Zero;
	}

	return NameResolves::Single(acumulator.iter().position(|&x| x == true).unwrap());
}

pub fn print_room(spot :&Place) {
	println!("{}", spot.desc);
	println!("{}", spot.long);
}

pub fn print_amb(spot :&Place) {
	match &spot.ambient {
		Some(x) => println!("{}", x),
		None => (),
	}
}