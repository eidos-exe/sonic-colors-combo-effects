#![feature(

    proc_macro_hygiene
)]
#![allow(
    unused_imports,
	unused_assignments,
	non_upper_case_globals,
	non_snake_case,
    clippy::borrow_interior_mutable_const
)]

#[macro_use]
extern crate lazy_static;

use {
    smash::{
        lua2cpp::*,
        phx::*,
        app::{sv_animcmd::*, lua_bind::*, *},
        lib::{lua_const::*, L2CValue, L2CAgent},
        hash40
    },
    smash_script::*,
    smashline::{*, Priority::*}
};

mod frame;
mod data;
mod config_loading;
mod hit_owner;
mod combo_state;
mod attack_group;
mod throw_credit;
mod pummel;

#[skyline::main(name = "sonic_colors_combo_effects")]
pub fn main() {
    frame::install();
}