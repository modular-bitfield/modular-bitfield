//! In this benchmark we compare the macro generated code for
//! setters and getters to some hand-written code for the same
//! data structure.
//!
//! We do a performance analysis for the getter and setter of
//! all fields of both structs.
//!
//! Also we test here that our hand-written code and the macro
//! generated code actually are semantically equivalent.
//! This allows us to further enhance the hand-written code
//! and to eventually come up with new optimization tricks
//! for the macro generated code while staying correct.

mod utils;

use utils::{
    handwritten::{Generated, Handwritten},
    *,
};

macro_rules! generate_cmp_benchmark_for {
    (
        test($test_name_get:ident, $test_name_set:ident) {
            fn $fn_get:ident($name_get:literal);
            fn $fn_set:ident($name_set:literal);
        }
    ) => {
        fn $test_name_get() {
            println!();
            compare($name_get, &Generated::new, |input| {
                repeat(|| {
                    black_box(input.$fn_get());
                });
            });
            compare($name_get, &Handwritten::new, |input| {
                repeat(|| {
                    black_box(input.$fn_get());
                });
            });
        }

        fn $test_name_set() {
            println!();
            compare($name_set, &Generated::new, |mut input| {
                repeat(|| {
                    black_box(black_box(&mut input).$fn_set(1));
                });
            });
            compare($name_set, &Handwritten::new, |mut input| {
                repeat(|| {
                    black_box(black_box(&mut input).$fn_set(1));
                });
            });
        }
    };
}
generate_cmp_benchmark_for!(
    test(bench_get_a, bench_set_a) {
        fn a("handwritten vs modular-bitfield - get_a");
        fn set_a("handwritten vs modular-bitfield - set_a");
    }
);
generate_cmp_benchmark_for!(
    test(bench_get_b, bench_set_b) {
        fn b("handwritten vs modular-bitfield - get_b");
        fn set_b("handwritten vs modular-bitfield - set_b");
    }
);
generate_cmp_benchmark_for!(
    test(bench_get_c, bench_set_c) {
        fn c("handwritten vs modular-bitfield - get_c");
        fn set_c("handwritten vs modular-bitfield - set_c");
    }
);
generate_cmp_benchmark_for!(
    test(bench_get_d, bench_set_d) {
        fn d("handwritten vs modular-bitfield - get_d");
        fn set_d("handwritten vs modular-bitfield - set_d");
    }
);
generate_cmp_benchmark_for!(
    test(bench_get_e, bench_set_e) {
        fn e("handwritten vs modular-bitfield - get_e");
        fn set_e("handwritten vs modular-bitfield - set_e");
    }
);
generate_cmp_benchmark_for!(
    test(bench_get_f, bench_set_f) {
        fn f("handwritten vs modular-bitfield - get_f");
        fn set_f("handwritten vs modular-bitfield - set_f");
    }
);

fn main() {
    bench_get_a();
    bench_get_b();
    bench_get_c();
    bench_get_d();
    bench_get_e();
    bench_get_f();
    bench_set_a();
    bench_set_b();
    bench_set_c();
    bench_set_d();
    bench_set_e();
    bench_set_f();
}
