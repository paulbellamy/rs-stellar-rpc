use std::error;
use std::process;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use serde_json::{Value, json};
use serde::de::Error;
use stellar_contract_host::{HostContext};
use warp::Filter;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum Requests {
    Call,
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

#[tokio::main]
async fn main() {
    // let context = HostContext::default();
    // eprintln!("Hello World!");
    // process::exit(0);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let call = warp::post()
        .and(warp::path("rpc"))
        .and(warp::body::json())
        .map(|request: Request|
            format!("{:?}", request)
        );

    warp::serve(call)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

