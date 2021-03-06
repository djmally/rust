// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "lib"]

// used by the rpass test

#[derive(Copy)]
pub struct Struct;

#[derive(Copy)]
pub enum Unit {
    UnitVariant,
    Argument(Struct)
}

#[derive(Copy)]
pub struct TupleStruct(pub uint, pub &'static str);

// used by the cfail test

#[derive(Copy)]
pub struct StructWithFields {
    foo: int,
}

#[derive(Copy)]
pub enum EnumWithVariants {
    EnumVariant,
    EnumVariantArg(int)
}

