use std::io::Cursor;

use std::error::Error;
use stellar_contract_env_host::{
    xdr::{Error as XdrError, ReadXdr, WriteXdr, Hash, ScVec, ScVal, VecM},
    ContractId, Host, Vm,
};

/// Deserialize an SCVec XDR object of SCVal arguments from the C++ side of the
/// bridge, instantiate a Host and VM with the provided WASM, invoke the
/// requested function in the WASM, and serialize an SCVal back into a return
/// value.
pub fn invoke_contract(
    contractId: &str,
    wasmBase64: &str,
    func: &str,
    argsXdrBase64: &Option<String>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let contractId: ContractId = ContractId(Hash::read_xdr(&mut Cursor::new(base64::decode(wasmBase64)?.as_slice()))?);
    let wasm = base64::decode(wasmBase64)?;
    let args: ScVec = match argsXdrBase64 {
        Some(a) => ScVec::read_xdr(&mut Cursor::new(base64::decode(a)?.as_slice()))?,
        None => vec![].try_into()?,
    };

    let mut host = Host::default();
    let vm = Vm::new(&host, contractId, wasm.as_slice())?;

    let res = vm.invoke_function(&mut host, func, &args)?;
    eprintln!("args: {:?}", args);
    eprintln!("res: {:?}", res);

    let mut ret_xdr_buf: Vec<u8> = Vec::new();
    res.write_xdr(&mut Cursor::new(&mut ret_xdr_buf))?;
    Ok(ret_xdr_buf)
}
