

const TABLE_SIZE:usize = 1048576;//2^20
const TABLE_SET_SIZE:u8 = 8;

struct Entry{
    key: u64, //zobrist hash of 
    value: i32
}

struct TableSet{
    entries: [Entry; TABLE_SET_SIZE as usize],
    oldest: u8
}


pub struct TranspositionTable{
    sets: [TableSet; TABLE_SIZE]

}

impl TranspositionTable {

    fn set_index(key: u64) -> usize{
        key as usize%usize::trailing_zeros(TABLE_SIZE) as usize
    }
    pub fn get(&self, key: u64) -> Option<i32>{
        let set = &self.sets[Self::set_index(key)];

        //check if this key is stored in the set and if so: return the value
        for entry in &set.entries{
            if key == entry.key{
                return Some(entry.value);
            }
        }
        return None;
    }

    pub fn insert(&mut self, key: u64, value: i32){

        let set = &mut self.sets[Self::set_index(key)];

        let oldest_entry = &mut set.entries[set.oldest as usize];

        oldest_entry.key = key;
        oldest_entry.value = value;

        set.oldest = (set.oldest + 1) % TABLE_SET_SIZE;
    }
}