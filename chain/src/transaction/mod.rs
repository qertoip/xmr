// Copyright 2018 Jean Pierre Dudey <jeandudey@hotmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod transaction;
mod transaction_prefix;

pub use self::transaction::{Transaction, SignatureType};
pub use self::transaction_prefix::TransactionPrefix;

mod tx_in;
mod tx_in_gen;
mod tx_in_to_key;
mod tx_in_to_script;
mod tx_in_to_script_hash;

pub use self::tx_in::TxIn;
pub use self::tx_in_gen::TxInGen;
pub use self::tx_in_to_key::TxInToKey;
pub use self::tx_in_to_script::TxInToScript;
pub use self::tx_in_to_script_hash::TxInToScriptHash;

mod tx_out;
mod tx_out_target;
mod tx_out_to_key;
mod tx_out_to_script;
mod tx_out_to_script_hash;

pub use self::tx_out::TxOut;
pub use self::tx_out_target::TxOutTarget;
pub use self::tx_out_to_key::TxOutToKey;
pub use self::tx_out_to_script::TxOutToScript;
pub use self::tx_out_to_script_hash::TxOutToScriptHash;
