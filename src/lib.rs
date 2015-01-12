#![feature(slicing_syntax)]
extern crate regex;

// identify custom modules
pub mod client;
mod connection;
pub mod info;
pub mod message;
mod reader;
mod utils;