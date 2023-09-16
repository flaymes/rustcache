use crate::memcached::storage;
use crate::memcached::storage::Record;
use crate::protocol::binary_codec::{BinaryRequest, BinaryResponse};
use crate::protocol::{binary, binary_codec};
use std::sync::Arc;

pub struct BinaryHandler {
    storage: Arc<storage::Storage>,
}

impl BinaryHandler {
    pub fn new(store: Arc<storage::Storage>) -> BinaryHandler {
        BinaryHandler { storage: store }
    }

    pub fn handle_request(
        &mut self,
        req: binary_codec::BinaryRequest,
    ) -> Option<binary_codec::BinaryResponse> {
        let request_header = req.get_header();
        let mut response_header = BinaryHandler::create_handler(request_header.opcode);

        let result = match req {
            BinaryRequest::Get(get_request) => {
                let record = self.storage.get(&get_request.key);
                let ret = match record {
                    None => None,
                    Some(storage::Record::Value(data)) => {
                        response_header.body_length = data.value.len() as u32 + 4;
                        response_header.cas = 1;
                        Some(binary_codec::BinaryResponse::Get(binary::GetResponse {
                            header: response_header,
                            flags: data.header.flags,
                            key: Vec::new(),
                            value: data.value,
                        }))
                    }
                    Some(storage::Record::Counter(data)) => None,
                };

                ret
            }
            BinaryRequest::GetQuietly(get_quietly_req) => None,
            BinaryRequest::GetKey(get_key_req) => None,
            BinaryRequest::GetKeyQuietly(get_key_quietly_req) => None,
            BinaryRequest::Set(set_req) => {
                let record = storage::ValueData::new(
                    set_req.key,
                    set_req.value,
                    set_req.header.cas,
                    set_req.flags,
                    set_req.expiration,
                );
                self.storage.set(storage::Record::Value(record));
                response_header.cas = 1;
                Some(binary_codec::BinaryResponse::Set(binary::SetResponse {
                    header: response_header,
                }))
            }
            BinaryRequest::Add(add_req) => None,
            BinaryRequest::Replace(replace_req) => None,
        };

        result
    }

    fn create_handler(cmd: u8) -> binary::ResponseHeader {
        binary::ResponseHeader {
            magic: binary::Magic::Response as u8,
            opcode: cmd,
            ..binary::ResponseHeader::default()
        }
    }
}
