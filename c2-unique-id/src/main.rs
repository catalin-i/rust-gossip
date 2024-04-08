use maelstrom_rs::{
    actor::Actor,
    error::Error,
    message::{Request, Response},
    runtime::Runtime,
};
use serde_json::{Map, Value};

fn main() {
    let node = UuidActor { node_id: None };
    let mut runtime = Runtime::new(Box::new(node));
    runtime.start();
}

struct UuidActor {
    node_id: Option<String>,
}

impl Actor for UuidActor {
    fn init(&mut self, node_id: &str, _node_ids: Vec<String>) -> Result<(), Error> {
        self.node_id = Some(String::from(node_id));
        eprintln!("node {} initialized", node_id);
        Ok(())
    }

    fn receive(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        match request.message_type.as_str() {
            "generate" => self.handle_generate(request),
            _ => unimplemented!("not implemented"),
        }
    }
}

impl UuidActor {
    pub(crate) fn handle_generate(&self, request: &Request) -> Result<Vec<Response>, Error> {
        let mut body = Map::new();
        let uuid = uuid::Uuid::new_v4().to_string();
        eprintln!("generated id: {}", uuid);
        body.insert("id".to_string(), Value::String(uuid));
        Ok(vec![Response::new_from_request(request, body)])
    }
}
