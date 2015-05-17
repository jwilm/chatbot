#![feature(std_misc)]
#![deny(unused_must_use)]

pub mod chatbot;
pub mod adapter;
pub mod handler;
pub mod message;

pub use chatbot::Chatbot;
