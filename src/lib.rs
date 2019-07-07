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

pub mod common;
pub mod engine;
pub mod frontends;
pub mod interfaces;
pub mod primitives;
