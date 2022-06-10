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
    contract_id_base64: &str,
    wasm_base64: &str,
    func: &str,
    args_xdr_base64: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let contract_id: ContractId = ContractId(Hash::read_xdr(&mut Cursor::new(base64::decode(contract_id_base64)?.as_slice()))?);
    let wasm = base64::decode(wasm_base64)?;
    let args: ScVec = if args_xdr_base64.len() > 0 {
        ScVec::read_xdr(
            &mut Cursor::new(
                base64::decode(args_xdr_base64)?.as_slice()
            )
        )?
    } else {
        vec![].try_into()?
    };

    let mut host = Host::default();
    let vm = Vm::new(&host, contract_id, wasm.as_slice())?;

    let res = vm.invoke_function(&mut host, func, &args)?;
    eprintln!("args: {:?}", args);
    eprintln!("res: {:?}", res);

    let mut ret_xdr_buf: Vec<u8> = Vec::new();
    res.write_xdr(&mut Cursor::new(&mut ret_xdr_buf))?;
    Ok(ret_xdr_buf)
}
