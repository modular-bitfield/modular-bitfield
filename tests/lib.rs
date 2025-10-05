#![no_std]
#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic, rust_2018_idioms)]
#![allow(dead_code)]
#![feature(const_trait_impl)]
#![feature(const_try)]
#![feature(derive_const)]
#![feature(const_index)]
#![feature(const_option_ops)]
#![feature(const_convert)] // TODO: this might be eliminated with some work

mod bitfield;
