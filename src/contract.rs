use std::io::Cursor;

use std::error::Error;
use stellar_contract_env_host::{
    xdr::{Error as XdrError, ReadXdr, WriteXdr, ScVec, ScVal, VecM},
    Host, Vm,
};

/// Deserialize an SCVec XDR object of SCVal arguments from the C++ side of the
/// bridge, instantiate a Host and VM with the provided WASM, invoke the
/// requested function in the WASM, and serialize an SCVal back into a return
/// value.
pub fn invoke_contract(
    wasmBase64: &str,
    func: &str,
    argsXdrBase64: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let wasm = base64::decode(wasmBase64)?;
    let args = base64::decode(argsXdrBase64)?;
    let arg_scvals = ScVec::read_xdr(&mut Cursor::new(args.as_slice()))?;

    let mut host = Host::default();
    let vm = Vm::new(&host, wasm.as_slice())?;

    let res = vm.invoke_function(&mut host, func, &arg_scvals)?;
    eprintln!("arg_scvals: {:?}", arg_scvals);
    eprintln!("res: {:?}", res);

    let mut ret_xdr_buf: Vec<u8> = Vec::new();
    res.write_xdr(&mut Cursor::new(&mut ret_xdr_buf))?;
    Ok(ret_xdr_buf)
}
