# documentation for tree.json

    struct Object { //represents item
        desc: String, //item discription
        can_take: bool, //true if the item can be picked up
        names: [String], //item names
    }

    struct Place {
        desc: String, //the first line of discription
        ambient: Option<String>, //amdiant noise (printed randomle)
        long: String, //second line of desc
        moves: {String: String}, //direction to room name pairs
        objects: [Object] //list of objects in room
    }

    struct World {
        map: {String: Place}, //name, room pairs
        location: String, //curent room
        aliases: {String: String}, //command aliases
        backpack: [Object], //list of items in backpack
    }

## tree.json

{String: Place}

## save games

World