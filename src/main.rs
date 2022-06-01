mod contract;
mod txmeta;
use txmeta::LedgerBackend;
use std::error::Error;
use std::process;
use std::rc::Rc;
use base64;
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use serde_json::{Value, json};
use stellar_xdr::{WriteXdr, ScVec, ScVal, VecM};
use warp::Filter;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum Requests {
    Call { contract: String, func: String, xdr: String, source_account: Option<String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum Notifications {
}

#[derive(Debug)]
enum JsonRpc<N, R> {
    Request(usize, R),
    Notification(N),
}

impl<N, R> Serialize for JsonRpc<N, R>
    where N: Serialize,
          R: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            JsonRpc::Request(id, ref r) => {
                let mut v = serde_json::to_value(r).map_err(serde::ser::Error::custom)?;
                v["id"] = json!(id);
                v.serialize(serializer)
            }
            JsonRpc::Notification(ref n) => n.serialize(serializer),
        }
    }
}

impl<'de, N, R> Deserialize<'de> for JsonRpc<N, R>
    where N: Deserialize<'de>,
          R: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct IdHelper {
            id: Option<usize>,
        }

        let v = Value::deserialize(deserializer)?;
        let helper = IdHelper::deserialize(&v).map_err(serde::de::Error::custom)?;
        match helper.id {
            Some(id) => {
                let r = R::deserialize(v).map_err(serde::de::Error::custom)?;
                Ok(JsonRpc::Request(id, r))
            }
            None => {
                let n = N::deserialize(v).map_err(serde::de::Error::custom)?;
                Ok(JsonRpc::Notification(n))
            }
        }
    }
}

type Request = JsonRpc<Notifications, Requests>;

// args.zig
const ARGS_WASM: &'static str = "AGFzbQEAAAABBgFgAX4BfgIRAQNlbnYJbG9nX3ZhbHVlAAADAgEABQMBAAEGCAF/AUGAgAQLBxMCBm1lbW9yeQIABmludm9rZQABCg8BDQAgABCAgICAABogAAs=";

// factorial.zig
const FACTORIAL_WASM: &'static str = "AGFzbQEAAAABBgFgAX4BfgMCAQAFAwEAAQYIAX8BQYCABAsHEwIGbWVtb3J5AgAGaW52b2tlAAAKNwE1AQJ+QgAhASAAQgAgAEIAVRshAkIBIQACQANAIAIgAVENASAAIAFCAXwiAX4hAAwACwsgAAs=";

// hello_world.zig
const HELLO_WORLD_WASM: &'static str = "AGFzbQEAAAABBQFgAAF/AwIBAAUDAQACBggBfwFBgIAECwcRAgZtZW1vcnkCAARyZWFkAAAKCgEIAEGAgISAAAsLFAEAQYCABAsMaGVsbG8gd29ybGQA
";

fn get_state() -> Result<(), Box<dyn Error>> {
    // TODO: Stream this later, so we don't have to do it on-the-fly.
    // Get the current state so we know the contract data, and can populate env.
    let backend = txmeta::FSLedgerBackend::default();
    // Get the latest checkpoint
    let latest = backend.get_latest()?;
    let checkpoint = latest/64; // +1 here?
    let replayFrom = checkpoint*64;
    let state = backend.get_checkpoint(checkpoint)?;
    // Get all ledgers after that, and replay them on top.
    for seq in replayFrom..(latest+1) {
        let ledger = backend.get_ledger(seq)?;
        // TODO: Replay the ledger into some in-memory state bucket
    };
    Ok(())
}

#[tokio::main]
async fn main() {
    // let context = HostContext::default();
    // eprintln!("Hello World!");
    // process::exit(0);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let call = warp::post()
        .and(warp::path("rpc"))
        .and(warp::body::json())
        .map(|request: Requests| match request {
            Requests::Call { contract: _, func, xdr, source_account: _ } => {
                get_state();

                // let v: ScVec = vec![ScVal::I32(1)].try_into().unwrap();
                // format!("xdr: {:?}", v.to_xdr_base64())

                // Invoke the contract
                match contract::invoke_contract(&FACTORIAL_WASM, &func, &xdr) {
                    Ok(result_xdr) => {
                        json!({
                            "result": base64::encode(result_xdr)
                        }).to_string()
                    }
                    Err(err) => {
                        json!({
                            "error": err.to_string()
                        }).to_string()
                    }
                }
            }
        });

    warp::serve(call)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

