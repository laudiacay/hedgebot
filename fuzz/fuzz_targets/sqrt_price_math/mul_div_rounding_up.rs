#![no_main]
use libfuzzer_sys::fuzz_target;
use hedgebot::solidints::U256;
use hedgebot::solidints::U160::U160;
use hedgebot::solidints::I256::I256;
use hedgebot::solidmath::full_math;
use hedgebot::solidmath::sqrt_price_math;
use hedgebot::solidmath::fixed_point;
use arbitrary;

#[derive(Debug)]
struct Args {
    x: U256,
    y: U256,
    z: U256,
}

impl arbitrary::Arbitrary<'_> for Args {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Self> {
        Ok( Args {
            x: U256(<[u64; 4]>::arbitrary(u)?),
            y: U256(<[u64; 4]>::arbitrary(u)?),
            z: U256(<[u64; 4]>::arbitrary(u)?),
        }
        )
    }
}

fuzz_target!(|data: Args| {
    let x = data.x;
    let y = data.y;
    let z = data.z;

    if !(z > U256::from(0 as i32)) { return; }
    let not_rounded_up = full_math::muldiv(x, y, z).unwrap();
    let rounded_up = full_math::mul_div_rounding_up(x, y, z).unwrap();
    assert!(rounded_up >= not_rounded_up);
    assert!(rounded_up - not_rounded_up < U256::from(2 as i32));
    if rounded_up - not_rounded_up == U256::from(1 as i32) {
        assert!(full_math::mulmod(x, y, z).unwrap() > U256::from(0 as i32));
    } else {
        assert!(full_math::mulmod(x, y, z).unwrap() == U256::from(0 as i32));
    }
});
