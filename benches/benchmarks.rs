#![allow(dead_code)]

mod utils;

use modular_bitfield::{
    bitfield,
    specifiers::{B12, B13, B16, B3, B32, B36, B4, B6, B7, B8, B9},
};
use utils::*;

#[bitfield]
pub struct Color {
    r: B8,
    g: B8,
    b: B8,
    a: B8,
}

#[bitfield]
pub struct SingleBitsInSingleByte {
    b0: bool,
    b1: bool,
    b2: bool,
    b3: bool,
    b4: bool,
    b5: bool,
    b6: bool,
    b7: bool,
}

#[bitfield]
pub struct TwoHalfBytes {
    h0: B4,
    h1: B4,
}

#[bitfield]
pub struct SingleBitAndRest {
    head: bool,
    rest: B7,
}

#[bitfield]
pub struct B7B1 {
    b7: B7,
    b1: bool,
}

#[bitfield]
pub struct B3B1B4 {
    b3: B3,
    b1: bool,
    b4: B4,
}

#[bitfield]
pub struct TwoHalfWords {
    fst: B16,
    snd: B16,
}

#[bitfield]
pub struct B6B12B6 {
    front: B6,
    middle: B12,
    back: B6,
}

#[bitfield]
pub struct B6B36B6 {
    front: B6,
    middle: B36,
    back: B6,
}

#[bitfield]
pub struct Complex {
    a: B9,  // 0th and 1st
    b: B6,  // 1st
    c: B13, // 1st, 2nd, 3rd
    d: B4,  // 3rd
    e: B32, // 4th, .., 7th
}

fn bench_set_variants() {
    one_shot("set - Color", &Color::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_r(1);
            black_box(&mut input).set_g(1);
            black_box(&mut input).set_b(1);
            black_box(&mut input).set_a(1);
        });
    });
    one_shot(
        "set - SingleBitsInSingleByte",
        &SingleBitsInSingleByte::new,
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_b0(true);
                black_box(&mut input).set_b1(true);
                black_box(&mut input).set_b2(true);
                black_box(&mut input).set_b3(true);
                black_box(&mut input).set_b4(true);
                black_box(&mut input).set_b5(true);
                black_box(&mut input).set_b6(true);
                black_box(&mut input).set_b7(true);
            });
        },
    );
    one_shot("set - TwoHalfBytes", &TwoHalfBytes::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_h0(1);
            black_box(&mut input).set_h1(1);
        });
    });
    one_shot(
        "set - SingleBitAndRest",
        &SingleBitAndRest::new,
        |mut input| {
            repeat(|| {
                black_box(&mut input).set_head(true);
                black_box(&mut input).set_rest(1);
            });
        },
    );
    one_shot("set - B7B1", &B7B1::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_b7(1);
            black_box(&mut input).set_b1(true);
        });
    });
    one_shot("set - B3B1B4", &B3B1B4::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_b3(1);
            black_box(&mut input).set_b1(true);
            black_box(&mut input).set_b4(1);
        });
    });
    one_shot("set - TwoHalfWords", &TwoHalfWords::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_fst(1);
            black_box(&mut input).set_snd(1);
        });
    });
    one_shot("set - B6B12B6", &B6B12B6::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_front(1);
            black_box(&mut input).set_middle(1);
            black_box(&mut input).set_back(1);
        });
    });
    one_shot("set - B6B36B6", &B6B36B6::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_front(1);
            black_box(&mut input).set_middle(1);
            black_box(&mut input).set_back(1);
        });
    });
    one_shot("set - Complex", &Complex::new, |mut input| {
        repeat(|| {
            black_box(&mut input).set_a(1);
            black_box(&mut input).set_b(1);
            black_box(&mut input).set_c(1);
            black_box(&mut input).set_d(1);
            black_box(&mut input).set_e(1);
        });
    });
}

fn bench_get_variants() {
    one_shot("get - Color", &Color::new, |input| {
        repeat(|| {
            black_box(input.r());
            black_box(input.g());
            black_box(input.b());
            black_box(input.a());
        });
    });
    one_shot(
        "get - SingleBitsInSingleByte",
        &SingleBitsInSingleByte::new,
        |input| {
            repeat(|| {
                black_box(input.b0());
                black_box(input.b1());
                black_box(input.b2());
                black_box(input.b3());
                black_box(input.b4());
                black_box(input.b5());
                black_box(input.b6());
                black_box(input.b7());
            });
        },
    );
    one_shot("get - TwoHalfBytes", &TwoHalfBytes::new, |input| {
        repeat(|| {
            black_box(input.h0());
            black_box(input.h1());
        });
    });
    one_shot("get - SingleBitAndRest", &SingleBitAndRest::new, |input| {
        repeat(|| {
            black_box(input.head());
            black_box(input.rest());
        });
    });
    one_shot("get - B7B1", &B7B1::new, |input| {
        repeat(|| {
            black_box(input.b7());
            black_box(input.b1());
        });
    });
    one_shot("get - B3B1B4", &B3B1B4::new, |input| {
        repeat(|| {
            black_box(input.b3());
            black_box(input.b1());
            black_box(input.b4());
        });
    });
    one_shot("get - TwoHalfWords", &TwoHalfWords::new, |input| {
        repeat(|| {
            black_box(input.fst());
            black_box(input.snd());
        });
    });
    one_shot("get - B6B12B6", &B6B12B6::new, |input| {
        repeat(|| {
            black_box(input.front());
            black_box(input.middle());
            black_box(input.back());
        });
    });
    one_shot("get - B6B36B6", &B6B36B6::new, |input| {
        repeat(|| {
            black_box(input.front());
            black_box(input.middle());
            black_box(input.back());
        });
    });
    one_shot("get - Complex", &Complex::new, |input| {
        repeat(|| {
            black_box(input.a());
            black_box(input.b());
            black_box(input.c());
            black_box(input.d());
            black_box(input.e());
        });
    });
}

fn main() {
    bench_set_variants();
    bench_get_variants();
}
