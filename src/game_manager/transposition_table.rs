
use crate::game_manager::ChessMove;

const TABLE_SIZE:usize = 1048576;//2^20
const TABLE_SET_SIZE:usize = 8;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct MoveEntry{
    chess_move: ChessMove,
    eval: i32
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct Entry{
    key: u64, //zobrist hash of boardstate
    value: [MoveEntry; 4]
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct TableSet{
    entries: [Entry; TABLE_SET_SIZE],
    oldest: u8
}

#[allow(dead_code)]
pub struct TranspositionTable{
    sets: [TableSet; TABLE_SIZE/TABLE_SET_SIZE]

}

#[allow(dead_code)]
impl TranspositionTable {

    pub fn new() -> Self{
        Self { 
            sets: [
                TableSet{
                    entries: [
                        Entry{
                            key: 0,
                            value: [MoveEntry{
                                chess_move: ChessMove::new_empty(),
                                eval: 0
                            }; 4]
                    }; TABLE_SET_SIZE],
                    oldest: 0
                };
                TABLE_SIZE/TABLE_SET_SIZE] 
        }
    }

    fn set_index(key: u64) -> usize{
        key as usize%usize::trailing_zeros(TABLE_SIZE) as usize
    }

    pub fn get(&self, key: u64) -> Option<[MoveEntry; 4]>{
        let set = &self.sets[Self::set_index(key)];

        //check if this key is stored in the set and if so: return the value
        for entry in &set.entries{
            if key == entry.key{
                return Some(entry.value);
            }
        }
        return None;
    }

    pub fn insert(&mut self, key: u64, value: [MoveEntry; 4]){

        let set = &mut self.sets[Self::set_index(key)];

        let oldest_entry = &mut set.entries[set.oldest as usize];

        oldest_entry.key = key;
        oldest_entry.value = value;

        set.oldest = (set.oldest + 1) % TABLE_SET_SIZE as u8;
    }
}

mod test{
    


    #[test]
    fn table_test(){
        
    }
}
