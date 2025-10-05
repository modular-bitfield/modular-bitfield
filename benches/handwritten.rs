#![allow(dead_code)]
#![feature(const_try)]
#![feature(const_trait_impl)]
#![feature(const_index)]

mod utils;
use utils::handwritten::{Generated, Handwritten};

macro_rules! impl_getter_setter_tests {
    ( $( ($name:ident, $getter:ident, $setter:ident, $n:expr), )* ) => {
        #[cfg(test)]
        mod generated_is_equal_to_handwritten {
            $(
                #[test]
                fn $name() {
                    let mut macro_struct = super::Generated::new();
                    let mut hand_struct = super::Handwritten::new();
                    assert_eq!(hand_struct.$getter(), macro_struct.$getter());
                    macro_struct.$setter($n);
                    hand_struct.$setter($n);
                    assert_eq!(hand_struct.$getter(), $n);
                    assert_eq!(macro_struct.$getter(), $n);
                    macro_struct.$setter(0);
                    hand_struct.$setter(0);
                    assert_eq!(hand_struct.$getter(), 0);
                    assert_eq!(macro_struct.$getter(), 0);
                }
            )*
        }
    }
}
impl_getter_setter_tests!(
    (get_set_a, a, set_a, 0b0001_1111_1111),
    (get_set_b, b, set_b, 0b0011_1111),
    (get_set_c, c, set_c, 0b0001_1111_1111_1111),
    (get_set_d, d, set_d, 0b0001),
    (get_set_e, e, set_e, 0b0111),
    (get_set_f, f, set_f, u32::MAX),
);
