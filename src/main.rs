mod contract;
use std::error;
use std::process;
use std::rc::Rc;
use base64;
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use serde_json::{Value, json};
use serde::de::Error;
use stellar_xdr::{WriteXdr, ScVec, ScVal, VecM};
use warp::Filter;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum Requests {
    Call { func: String, xdr: String },
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
        let helper = IdHelper::deserialize(&v).map_err(Error::custom)?;
        match helper.id {
            Some(id) => {
                let r = R::deserialize(v).map_err(Error::custom)?;
                Ok(JsonRpc::Request(id, r))
            }
            None => {
                let n = N::deserialize(v).map_err(Error::custom)?;
                Ok(JsonRpc::Notification(n))
            }
        }
    }
}

type Request = JsonRpc<Notifications, Requests>;

// factorial.ts
const wasm: &'static str = "AGFzbQEAAAABFwRgAAF+YAF+AX5gA35+fgF+YAJ+fgF+AnwGA2Vudglsb2dfdmFsdWUAAQNlbnYHbWFwX25ldwAAA2VudgdtYXBfcHV0AAIDZW52B21hcF9nZXQAAwNlbnYWZ2V0X2N1cnJlbnRfbGVkZ2VyX251bQAAA2Vudh1nZXRfY3VycmVudF9sZWRnZXJfY2xvc2VfdGltZQAAAwIBAQUDAQAAB3YICWxvZ192YWx1ZQAAB21hcF9uZXcAAQdtYXBfcHV0AAIHbWFwX2dldAADFmdldF9jdXJyZW50X2xlZGdlcl9udW0ABB1nZXRfY3VycmVudF9sZWRnZXJfY2xvc2VfdGltZQAFBmludm9rZQAGBm1lbW9yeQIACi0BKwECf0EBIQJBASEBA0AgACABrFkEQCABIAJsIQIgAUEBaiEBDAELCyACrAs=";

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
            Requests::Call { func, xdr } => {
                let wasmBytes = base64::decode(wasm);
                let args = base64::decode(xdr);
                // let mut v = Vec::<ScVal>::new();
                // v.push(ScVal::ScvI32(1));
                // let v: ScVec = ScVec(v.try_into().unwrap());
                // format!("xdr: {:?}", v.to_xdr_base64())
                match (wasmBytes, args) {
                    (Ok(w), Ok(a)) => {
                        let result = contract::invoke_contract(&w, &func, &a);
                        format!("Result: {:?}", result)
                    }
                    (Err(e), _) => {
                        format!("Failed to parse wasm: {:?}", e)
                    }
                    (_, Err(e)) => {
                        format!("Failed to parse args: {:?}", e)
                    }
                }
            }
        });

    warp::serve(call)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

