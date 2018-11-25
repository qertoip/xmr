// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use uint::U256;
use uint::U512;

lazy_static! {
    static ref U256_MAX: U512 = U512::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").expect("to be correct unsigned integer");
    //                                              ^^^^^^^^^^ 2^256 - 1 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
}

pub fn proof_of_work_is_valid(pow_bytes_le: &[u8], difficulty: u64) -> bool {
    let pow_u256 = U256::from_little_endian(pow_bytes_le);
    let difficulty_u256 = U256::from(difficulty);
    pow_u256.full_mul(difficulty_u256) <= *U256_MAX
}

#[cfg(test)]
mod tests {
    extern crate hex;
    extern crate time;
    extern crate rand;

    use super::*;
    use self::rand::*;
    use self::time::{PreciseTime, Duration};

    #[test]
    fn mainnet_genesis_pow_is_valid()  /* https://xmrchain.net/block/0 */  {
        assert_valid_pow_hex("8a7b1a780e99eec31a9425b7d89c283421b2042a337d5700dfd4a7d6eb7bd774", 1);
    }

    #[test]
    fn mainnet_pow_at_1_is_valid()  /* https://xmrchain.net/block/1 */  {
        assert_valid_pow_hex("5aeebb3de73859d92f3f82fdb97286d81264ecb72a42e4b9f1e6d62eb682d7c0", 1);
    }

    #[test]
    fn mainnet_pow_at_3_is_valid()  /* https://xmrchain.net/block/3 */  {
        assert_valid_pow_hex("146f7a7ccafd32eed8f1bfefe73a69d19c655ea1ce005fcb9045fa963a0c0101", 60);
    }

    #[test]
    fn mainnet_pow_at_1708472_is_valid()  /* https://xmrchain.net/block/1708472 */ {
        assert_valid_pow_hex("baa3060d1725b71cc9018877f488eeff7633ce514f2097c7907d1a1300000000", 51638511039);
    }

    #[test]
    fn pow_precisely_at_target_is_valid() {
        // target is a function of difficulty as defined in https://cryptonote.org/cns/cns010.txt
        // target = floor((2^256-1) / difficulty)
        let difficulty = 51638511039;  // example difficulty - real world value for Nov 2018
        let target = U256::from_dec_str("2242359179370299570181822279337156699950563511941089607981823668320").expect("to be correct uns int");
        let pow = target;
        assert_valid_pow_u256(pow, difficulty);
    }

    #[test]
    fn pow_at_target_plus_one_is_invalid() {
        // target is a function of difficulty as defined in https://cryptonote.org/cns/cns010.txt
        // target = floor((2^256-1) / difficulty)
        let difficulty = 51638511039;
        let target = U256::from_dec_str("2242359179370299570181822279337156699950563511941089607981823668320").expect("to be correct uns int");
        let pow = target + 1_u64;
        assert_invalid_pow_u256(pow, difficulty);
    }

    #[test]
    fn arbitrary_pow_is_invalid() {
        // target is a function of difficulty as defined in https://cryptonote.org/cns/cns010.txt
        // target = floor((2^256-1) / difficulty)
        let difficulty = 51638511039;
        let pow = "8a085dfc3e5bef71f611d372d8c0040e9a525f08b9b53de9f0804946218e0fb8";
        assert_invalid_pow_hex(pow, difficulty);
    }

    #[test]
    fn max_pow_is_invalid() {
        // target is a function of difficulty as defined in https://cryptonote.org/cns/cns010.txt
        // target = floor((2^256-1) / difficulty)
        let difficulty = 2;
        let pow = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_invalid_pow_hex(pow, difficulty);
    }

    #[test]
    fn bench_10k_pow_checks_under_100ms() {
        let n: u64 = 10_000;
        let difficulty: u64 = 51638511039;
        let rb = random_32_bytes();
        let pows: Vec<Vec<u8>> = (0..n).map(|_| { rb.clone() }).collect();

        let start = PreciseTime::now();
        let _: Vec<bool> = pows.iter().map(|pow| { proof_of_work_is_valid(&pow, difficulty) }).collect();
        let end = PreciseTime::now();
        let duration = start.to(end);
        assert!(duration < Duration::milliseconds(100));
        //println!("duration = {}s", duration);
    }

    fn assert_valid_pow_hex(pow_hex_le: &str, difficulty: u64) {
        let pow_bytes_le = hex::decode(pow_hex_le).expect("to be correct hex");
        assert!(proof_of_work_is_valid(&pow_bytes_le, difficulty));
    }

    fn assert_invalid_pow_hex(pow_hex_le: &str, difficulty: u64) {
        let pow_bytes_le = hex::decode(pow_hex_le).expect("to be correct hex");
        assert!(!proof_of_work_is_valid(&pow_bytes_le, difficulty));
    }

    fn assert_valid_pow_u256(pow_u256_le: U256, difficulty: u64) {
        let mut pow_bytes_le: [u8; 32] = [0; 32];
        pow_u256_le.to_little_endian(&mut pow_bytes_le);
        assert!(proof_of_work_is_valid(&pow_bytes_le, difficulty));
    }

    fn assert_invalid_pow_u256(pow_u256_le: U256, difficulty: u64) {
        let mut pow_bytes_le: [u8; 32] = [0; 32];
        pow_u256_le.to_little_endian(&mut pow_bytes_le);
        assert!(!proof_of_work_is_valid(&pow_bytes_le, difficulty));
    }

    fn random_32_bytes() -> Vec<u8> {
        (0..32).map(|_| { random::<u8>() }).collect()
    }
}
