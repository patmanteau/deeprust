use std::fmt;
use moves::{Move, UnmakeInfo};

#[derive(Copy, Clone, Debug)]
pub struct MoveStackEntry {
    pub mov: Move,
    pub store: UnmakeInfo,
}

impl MoveStackEntry {
    pub fn new(mov: Move, store: UnmakeInfo) -> MoveStackEntry {
        MoveStackEntry {
            mov: mov,
            store: store,
        }
    }

    fn empty() -> MoveStackEntry {
        MoveStackEntry {
            mov: Move::new(0, 0, 0),
            store: UnmakeInfo::new(0, 0, [0, 0], 0, false, 0),
        }
    }
}

impl fmt::Display for MoveStackEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mov)
    }
}

/// Keeps a move history for unmaking moves
#[derive(Clone, Debug)]
pub struct MoveStack {
    entries: Vec<MoveStackEntry>,
    current: usize,
}

impl MoveStack {

    /// Constructs a new, empty MoveStack
    #[inline]
    pub fn new() -> MoveStack {
        MoveStack {
            entries: Vec::with_capacity(1024),
            current: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, entry: MoveStackEntry) {
        // self.entries[self.current] = entry;
        // self.current += 1;
        self.entries.push(entry)
    }

    #[inline]
    pub fn pop(&mut self) -> MoveStackEntry {
        // debug_assert!(0 < self.current);
        // let retval = self.entries[self.current - 1];
        // self.current -= 1;
        // retval
        self.entries.pop().unwrap()
    }

    #[inline]
    pub fn peek(&self) -> &MoveStackEntry {
        // debug_assert!(0 < self.current);
        // let retval = self.entries[self.current - 1];
        // // self.current -= 1;
        // retval
        self.entries.last().unwrap()
    }

    #[inline]
    pub fn len(&self) -> usize {
        // self.current
        self.entries.len()
    }
}

impl fmt::Display for MoveStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        for mov in self.entries.iter() {
            str.push_str(&format!("{} ", mov));
        }
        write!(f, "{}", str)
    }
}

