use std::default;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use std::num::ParseIntError;
use std::str::FromStr;

use stellar_contract_env_host::{
    xdr::{Error as XdrError, ReadXdr, WriteXdr, ScVec, ScVal, VecM, LedgerCloseMeta},
    Host, Vm,
};

pub trait LedgerBackend {
    fn get_checkpoint(&self, checkpoint: u32) -> Result<u32, Box<dyn Error>>;
    fn get_latest(&self) -> Result<u32, Box<dyn Error>>;
    fn get_ledger(&self, seq: u32) -> Result<LedgerCloseMeta, Box<dyn Error>>;
}

#[derive(Debug, PartialEq)]
pub struct FSLedgerBackend<'a> {
    root: &'a str,
}

impl<'a> Default for FSLedgerBackend<'a> {
    fn default() -> Self {
        Self { root: "." }
    }
}

// TODO: Implement these for realsies
impl<'a> LedgerBackend for FSLedgerBackend<'a> {
    fn get_checkpoint(&self, checkpoint: u32) -> Result<u32, Box<dyn Error>> {
        panic!("TODO: Implement get_checkpoint");
    }

    fn get_latest(&self) -> Result<u32, Box<dyn Error>> {
        match fs::read_to_string("latest") {
            Ok(r) => {
                u32::from_str_radix(&r, 10)
                    .map_err(|e| From::from(e))
            },
            Err(err) => {
                 match err.kind() {
                    std::io::ErrorKind::NotFound => Ok(2),
                    _ => Err(From::from(err))
                }
            },
        }
    }

    fn get_ledger(&self, seq: u32) -> Result<LedgerCloseMeta, Box<dyn Error>> {
        let r = fs::read_to_string(format!("ledgers/{:}", seq))?;
        LedgerCloseMeta::read_xdr(&mut Cursor::new(r)).map_err(|e| From::from(e))
    }
}
