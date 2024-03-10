use maelstrom_rs::actor::Actor;
use maelstrom_rs::error::Error;
use maelstrom_rs::message::{Request, Response};
use maelstrom_rs::runtime::Runtime;
use serde_json::{Map, Value};

fn main() {
    let node = BroadcastActor {
        node_id: None,
        node_ids: vec![],
        neighbours: vec![],
        messages: vec![],
    };
    let mut runtime = Runtime::new(Box::new(node));
    runtime.start();
}

struct BroadcastActor {
    node_id: Option<String>,
    node_ids: Vec<String>,
    neighbours: Vec<String>,
    messages: Vec<Value>,
}

impl Actor for BroadcastActor {
    fn init(&mut self, node_id: &str, node_ids: Vec<String>) -> Result<(), Error> {
        self.node_id = Some(node_id.to_string());
        self.node_ids = node_ids;
        eprintln!("Node {} started!", node_id);
        Ok(())
    }

    fn receive(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        match request.message_type.as_str() {
            "broadcast" => self.handle_broadcast(request),
            "read" => self.handle_read(request),
            "topology" => self.handle_topology(request),
            _ => unimplemented!("no impl for type"),
        }
    }
}

impl BroadcastActor {
    pub(crate) fn handle_broadcast(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        let mut responses = vec![];

        let value = match request.body.get("message") {
            Some(value) => value,
            _ => unreachable!(),
        };

        if !self.messages.contains(value) {
            self.messages.push(value.clone());

            for neighbor in &self.neighbours {
                let mut body = Map::new();
                body.insert(String::from("message"), value.clone());
                responses.push(Response {
                    destination: neighbor.to_string(),
                    message_type: String::from("broadcast"),
                    message_id: None,
                    in_reply_to: None,
                    body
                });
            }
        }

        if request.message_id.is_some() {
            responses.push(Response::new_from_request(request, Default::default()))
        }
        Ok(responses)
    }

    pub(crate) fn handle_read(&self, request: &Request) -> Result<Vec<Response>, Error> {
        let mut body = Map::new();
        body.insert("messages".to_string(), Value::from(self.messages.clone()));
        Ok(vec![Response::new_from_request(request, body)])
    }

    pub(crate) fn handle_topology(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        let body = Map::new();
        let topology = match request.body.get("topology") {
            Some(Value::Object(t)) => t,
            _ => unreachable!(),
        };

        self.neighbours = match topology.get(self.node_id.as_ref().unwrap()) {
            Some(Value::Array(n)) => n
                .iter()
                .filter_map(|s| s.as_str())
                .map(String::from)
                .collect(),
            _ => return Err(Error::CustomError((1001, String::from("bad topology")))),
        };

        Ok(vec![Response::new_from_request(request, body)])
    }
}
