// #![warn(clippy::all)]
// #![feature(test)]
// #![feature(simd_x86_bittest)]

// #[macro_use]
// extern crate lazy_static;
// #[macro_use]
// extern crate log;
// extern crate nom;
// extern crate quanta;
// extern crate rand;
// extern crate rayon;
// // extern crate simple_logging;
// extern crate test;

mod castling;
mod color;
mod r#move;
mod piece;
mod square;

pub use castling::{sides, Castling, CastlingSide};
pub use color::{colors, Color, ColorPrimitives};
pub use r#move::{flags, Move, MoveStack};
pub use piece::{pieces, Piece, PiecePrimitives};
pub use square::{squares, Square, SquarePrimitives, ep_capture_square};
