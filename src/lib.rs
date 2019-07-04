#![warn(clippy::all)]
#![feature(test)]
#![feature(simd_x86_bittest)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nom;
extern crate quanta;
extern crate rand;
extern crate rayon;
// extern crate simple_logging;
extern crate test;

pub mod bitboard;
pub mod board;
pub mod cli;
pub mod color;
pub mod common;
pub mod interfaces;
pub mod move_generator;
pub mod moves;
pub mod piece;
pub mod position;
pub mod search;
pub mod square;
pub mod uci;
