use maelstrom_rs::actor::Actor;
use maelstrom_rs::error::Error;
use maelstrom_rs::message::{Request, Response};
use maelstrom_rs::runtime::{Event, Runtime};
use rand::{thread_rng, Rng};
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

fn main() {
    let node = BroadcastActor {
        sender: None,
        node_id: None,
        node_ids: vec![],
        neighbours: vec![],
        messages: HashSet::new(),
    };
    let mut runtime = Runtime::new(Box::new(node));
    runtime.start();
}

struct BroadcastActor {
    sender: Option<Sender<Event>>,
    node_id: Option<String>,
    node_ids: Vec<String>,
    neighbours: Vec<String>,
    messages: HashSet<i64>,
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
            "gossip" => self.handle_gossip(request),
            _ => unimplemented!("no impl for type"),
        }
    }

    fn gossip(&mut self) -> Result<Vec<Response>, Error> {
        let mut responses = vec![];
        let len = self.neighbours.len();
        let mut rng = thread_rng();
        let neigh_index = rng.gen_range(0..len);
        let node_index = rng.gen_range(0..self.node_ids.len());
        let neigh_dest = self.neighbours.get(neigh_index).unwrap();
        let node_dest = format!("n{}", node_index);
        self.send_to_node(&neigh_dest.clone(), &mut responses);
        self.send_to_node(&node_dest, &mut responses);
        Ok(responses)
    }

    fn inject_sender(&mut self, sender: Sender<Event>) {
        self.sender.replace(sender);
    }
}

impl BroadcastActor {
    pub(crate) fn handle_broadcast(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        let mut responses = vec![];

        let value = match request.body.get("message") {
            Some(value) => value.as_i64().expect("not a correct value"),
            _ => unreachable!(),
        };

        if !self.messages.contains(&value) {
            self.messages.replace(value);
        }

        if request.message_id.is_some() {
            responses.push(Response::new_from_request(request, Default::default()))
        }
        Ok(responses)
    }

    pub(crate) fn handle_read(&self, request: &Request) -> Result<Vec<Response>, Error> {
        let mut body = Map::new();
        body.insert(
            "messages".to_string(),
            Value::from(
                self.messages
                    .iter()
                    .map(|val| Value::from(*val))
                    .collect::<Vec<_>>(),
            ),
        );
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

        let sender = self.sender.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(50));
            sender
                .as_ref()
                .expect("no sender")
                .send(Event::Trigger)
                .expect("unable to send trigger");
        });

        Ok(vec![Response::new_from_request(request, body)])
    }

    pub(crate) fn handle_gossip(&mut self, request: &Request) -> Result<Vec<Response>, Error> {
        eprintln!("Received gossip!");
        let values = match request.body.get("messages") {
            Some(value) => value.as_array().expect("not an array"),
            _ => unreachable!(),
        };
        let values: HashSet<i64> = values
            .iter()
            .map(|val| val.as_i64().expect("not an i64"))
            .collect();
        self.messages.extend(values);
        Ok(vec![])
    }

    fn send_to_node(&mut self, dest: &str, responses: &mut Vec<Response>) {
        let mut body = Map::new();
        body.insert(
            "messages".to_string(),
            Value::from(
                self.messages
                    .iter()
                    .map(|val| Value::from(*val))
                    .collect::<Vec<_>>(),
            ),
        );
        eprintln!("pushing gossip!");
        responses.push({
            Response {
                source: self.node_id.clone().unwrap(),
                destination: dest.to_string(),
                message_type: "gossip".to_string(),
                message_id: None,
                in_reply_to: None,
                body,
            }
        });
    }
}
