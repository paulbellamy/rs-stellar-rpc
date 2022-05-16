use std::io::Cursor;

use std::error::Error;
use stellar_contract_env_host::{
    xdr::{ReadXDR, ScVec, WriteXDR},
    Host, VM,
};

/// Deserialize an SCVec XDR object of SCVal arguments from the C++ side of the
/// bridge, instantiate a Host and VM with the provided WASM, invoke the
/// requested function in the WASM, and serialize an SCVal back into a return
/// value.
pub fn invoke_contract(
    wasm: &Vec<u8>,
    func: &String,
    args: &Vec<u8>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let arg_scvals = ScVec::read_xdr(&mut Cursor::new(args.as_slice()))?;

    let mut host = Host::default();
    let vm = VM::new(&host, wasm.as_slice())?;

    let res = vm.invoke_function(&mut host, func, &arg_scvals)?;

    let mut ret_xdr_buf: Vec<u8> = Vec::new();
    res.write_xdr(&mut Cursor::new(&mut ret_xdr_buf))?;
    Ok(ret_xdr_buf)
}
