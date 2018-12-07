// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate uint;

mod pow_verification;

pub use pow_verification::{proof_of_work_is_valid};
