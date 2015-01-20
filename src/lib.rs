#![feature(slicing_syntax)]
#![allow(unstable)]
extern crate regex;

// identify custom modules
pub mod client;
pub mod connection;
pub mod ctcp;
pub mod info;
pub mod message;
pub mod reader;
mod utils;