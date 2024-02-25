use maelstrom_rs::{
    actor::Actor,
    error::Error,
    message::{Request, Response},
    runtime::Runtime,
};
use serde_json::{Map, Value};

fn main() {
    let node = EchoActor { node_id: None };
    let mut runtime = Runtime::new(Box::new(node));
    runtime.start();
}

struct EchoActor {
    node_id: Option<String>,
}

impl Actor for EchoActor {
    fn init(
        &mut self,
        node_id: &str,
        node_ids: Vec<String>,
    ) -> Result<(), maelstrom_rs::error::Error> {
        self.node_id = Some(String::from(node_id));
        eprintln!("node {} initiated", node_id);
        Ok(())
    }

    fn receive(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        match request.message_type.as_str() {
            "echo" => self.handle_echo(request),
            _ => unimplemented!(
                "unimplemented message type {}",
                request.message_type.as_str()
            ),
        }
    }
}

impl EchoActor {
    pub(crate) fn handle_echo(&self, request: &Request) -> Result<Vec<Response>, Error> {
        let echo = request.body.get("echo").unwrap().as_str().unwrap();
        let mut body = Map::new();

        body.insert("echo".to_string(), Value::from(String::from(echo)));
        Ok(vec![Response::new_from_request(request, body)])
    }
}
