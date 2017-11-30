use super::moves::{Move, UnmakeInfo};

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
            mov: Move::new(0, 0, 0, 0, 0),
            store: UnmakeInfo::new(0, 0, 0, 0, false, 0),
        }
    }
}

/// Keeps a move history for unmaking moves
#[derive(Copy, Clone)]
pub struct MoveStack {
    entries: [MoveStackEntry; 2048],
    current: usize,
}

impl MoveStack {

    /// Constructs a new, empty MoveStack
    #[inline]
    pub fn new() -> MoveStack {
        MoveStack {
            entries: [MoveStackEntry::empty(); 2048],
            current: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, entry: MoveStackEntry) {
        self.entries[self.current] = entry;
        self.current += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> MoveStackEntry {
        debug_assert!(0 < self.current);
        let retval = self.entries[self.current - 1];
        self.current -= 1;
        retval
    }

    #[inline]
    pub fn peek(&self) -> MoveStackEntry {
        debug_assert!(0 < self.current);
        let retval = self.entries[self.current - 1];
        // self.current -= 1;
        retval
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.current
    }
}

