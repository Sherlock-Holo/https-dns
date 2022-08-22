use rand::{thread_rng, Rng};
use trust_dns_proto::{
    op::{Message, MessageType, Query},
    rr::{Name, RecordType},
};

pub fn build_request_message(name: Name, record_type: RecordType) -> Message {
    let mut request_message = Message::new();

    request_message
        .set_id(thread_rng().gen_range(0..=65535))
        .set_message_type(MessageType::Query)
        .set_recursion_desired(true)
        .add_query(Query::query(name, record_type));

    request_message
}
