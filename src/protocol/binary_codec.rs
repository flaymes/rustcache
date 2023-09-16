use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BufMut, BytesMut};
use num_traits::FromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

use crate::protocol::binary;

/// Client request
#[derive(Serialize, Deserialize, Debug)]
pub enum BinaryRequest {
    Get(binary::GetRequest),
    GetQuietly(binary::GetQuietRequest),
    GetKey(binary::GetKeyRequest),
    GetKeyQuietly(binary::GetKeyQuietRequest),
    Set(binary::SetRequest),
    Add(binary::AddRequest),
    Replace(binary::ReplaceRequest),
}

impl BinaryRequest {
    pub fn get_header(&self) -> &binary::RequestHeader {
        match self {
            BinaryRequest::Get(request) => &request.header,
            BinaryRequest::GetQuietly(request) => &request.header,
            BinaryRequest::GetKey(request) => &request.header,
            BinaryRequest::GetKeyQuietly(request) => &request.header,
            BinaryRequest::Set(request) => &request.header,
            BinaryRequest::Add(request) => &request.header,
            BinaryRequest::Replace(request) => &request.header,
        }
    }
}

/// Server response
#[derive(Serialize, Deserialize, Debug)]
pub enum BinaryResponse {
    Get(binary::GetResponse),
    GetQuietly(binary::GetQuietlyResponse),
    GetKey(binary::GetKeyResponse),
    GetKeyQuietly(binary::GetKeyQuietlyResponse),
    Set(binary::SetResponse),
    Add(binary::AddResponse),
    Replace(binary::ReplaceResponse),
}

impl BinaryResponse {
    pub fn get_header(&self) -> &binary::ResponseHeader {
        match self {
            BinaryResponse::Get(response) => &response.header,
            BinaryResponse::GetQuietly(response) => &response.header,
            BinaryResponse::GetKey(response) => &response.header,
            BinaryResponse::GetKeyQuietly(response) => &response.header,
            BinaryResponse::Set(response) => &response.header,
            BinaryResponse::Add(response) => &response.header,
            BinaryResponse::Replace(response) => &response.header,
        }
    }
}

#[derive(PartialEq, Debug)]
enum RequestParserState {
    None,
    HeaderParsed,
    RequestParsed,
}

pub struct MemcachedBinaryCodec {
    header: binary::RequestHeader,
    state: RequestParserState,
}

impl MemcachedBinaryCodec {
    const HEADER_LEN: usize = 24;
    pub fn new() -> MemcachedBinaryCodec {
        MemcachedBinaryCodec {
            header: binary::RequestHeader {
                magic: 0,
                opcode: 0,
                key_length: 0,
                extras_length: 0,
                data_type: 0,
                reserved: 0,
                body_length: 0,
                opaque: 0,
                cas: 0,
            },
            state: RequestParserState::None,
        }
    }

    pub fn parse_header(&mut self, src: &mut BytesMut) {
        assert!(src.len() >= MemcachedBinaryCodec::HEADER_LEN);
        self.header = binary::RequestHeader {
            magic: src.get_u8(),
            opcode: src.get_u8(),
            key_length: src.get_u16(),
            extras_length: src.get_u8(),
            data_type: src.get_u8(),
            reserved: src.get_u16(),
            body_length: src.get_u32(),
            opaque: src.get_u32(),
            cas: src.get_u64(),
        };
        self.state = RequestParserState::HeaderParsed;
    }

    pub fn get_req_length(&self) -> usize {
        (self.header.extras_length as usize)
            + (self.header.key_length as usize)
            + (self.header.body_length as usize)
    }

    pub fn parse(&mut self, src: &mut BytesMut) -> Option<BinaryRequest> {
        assert!(src.len() >= self.get_req_length());
        assert_eq!(self.state, RequestParserState::RequestParsed);

        let result = match FromPrimitive::from_u8(self.header.opcode) {
            Some(binary::Command::Get) => {
                let size = self.header.key_length as usize;
                let buf = src.split_to(size);
                let key = buf.to_vec();

                Some(BinaryRequest::Get(binary::GetRequest {
                    header: self.header,
                    key: key,
                }))
            }
            Some(binary::Command::GetQuiet) => None,
            Some(binary::Command::GetKey) => None,
            Some(binary::Command::Flush) => None,
            Some(binary::Command::Append) => None,
            Some(binary::Command::Prepend) => None,
            Some(binary::Command::Set) => {
                let extras_size = self.header.extras_length;
                assert_eq!(extras_size, 8);
                assert_ne!(self.header.key_length, 0);
                assert!(self.header.body_length >= (self.header.key_length + 8) as u32);
                assert!(src.len() >= (self.header.body_length) as usize);

                let value_len = self.get_value_len();

                let set_request = binary::SetRequest {
                    header: self.header,
                    flags: BigEndian::read_u32(&src),
                    expiration: BigEndian::read_u32(&src),
                    key: src.split_to(self.header.key_length as usize).to_vec(),
                    value: src.split_to(value_len).to_vec(),
                };
                Some(BinaryRequest::Set(set_request))
            }
            Some(binary::Command::Add) => None,
            Some(binary::Command::Replace) => None,
            Some(binary::Command::Delete) => None,
            Some(binary::Command::Increment) => None,
            Some(binary::Command::Decrement) => None,
            Some(binary::Command::Quit) => None,
            Some(binary::Command::QuitQuiet) => None,
            Some(binary::Command::Noop) => None,
            Some(binary::Command::Version) => None,
            Some(binary::Command::GetKeyQuiet) => None,
            Some(binary::Command::Stat) => None,
            Some(binary::Command::SetQuiet) => None,
            Some(binary::Command::AddQuiet) => None,
            Some(binary::Command::ReplaceQuiet) => None,
            Some(binary::Command::DeleteQuiet) => None,
            Some(binary::Command::IncrementQuiet) => None,
            Some(binary::Command::DecrementQuiet) => None,
            Some(binary::Command::FlushQuiet) => None,
            Some(binary::Command::AppendQuiet) => None,
            Some(binary::Command::PrependQuiet) => None,
            Some(binary::Command::Touch) => None,
            Some(binary::Command::GetAndTouch) => None,
            Some(binary::Command::GetAndTouchQuiet) => None,
            Some(binary::Command::GetAndTouchKey) => None,
            Some(binary::Command::GetAndTouchKeyQuiet) => None,
            Some(binary::Command::SaslListMechs) => None,
            Some(binary::Command::SaslAuth) => None,
            Some(binary::Command::SaslStep) => None,
            None => {
                println!("Cannot parse command opcode {:?}", self.header);
                None
            }
        };

        self.state = RequestParserState::None;

        result
    }

    fn get_value_len(&self) -> usize {
        (self.header.body_length as usize) - ((self.header.key_length + 8) as usize)
    }
}

impl Decoder for MemcachedBinaryCodec {
    type Item = BinaryRequest;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.state {
            RequestParserState::None => {
                if src.len() < MemcachedBinaryCodec::HEADER_LEN {
                    return Ok(None);
                }
                self.parse_header(src);
            }
            RequestParserState::HeaderParsed => {
                if src.len() < self.get_req_length() {
                    return Ok(None);
                }
                return Ok(self.parse(src));
            }
            RequestParserState::RequestParsed => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data"));
            }
        }
        return Ok(None);
    }
}

impl MemcachedBinaryCodec {
    const RESPONSE_HEADER_LEN: usize = 24;
    fn get_length(&self, msg: &BinaryResponse) -> usize {
        self.get_len_from_header(self.get_header(msg))
    }

    fn get_header<'a>(&self, msg: &'a BinaryResponse) -> &'a binary::ResponseHeader {
        msg.get_header()
    }

    fn get_len_from_header(&self, header: &binary::ResponseHeader) -> usize {
        MemcachedBinaryCodec::RESPONSE_HEADER_LEN + (header.body_length as usize)
    }

    fn write_msg(&self, msg: &BinaryResponse, dst: &mut BytesMut) {}

    fn write_header(&self, header: &binary::ResponseHeader, dst: &mut BytesMut) {
        dst.put_u8(header.magic);
        dst.put_u8(header.opcode);
        dst.put_u16(header.key_length);
        dst.put_u8(header.extras_length);
        dst.put_u8(header.data_type);
        dst.put_u16(header.status);
        dst.put_u32(header.body_length);
        dst.put_u64(header.cas);
    }

    fn write_data(&self, msg: &BinaryResponse, dst: &mut BytesMut) {
        match msg {
            BinaryResponse::Get(response) => {
                dst.put_u32(response.flags);
                dst.put_slice(&response.key[..]);
                dst.put_slice(&response.value[..])
            }
            BinaryResponse::GetQuietly(response) => {
                dst.put_u32(response.flags);
                dst.put_slice(&response.key[..]);
                dst.put_slice(&response.value[..])
            }
            BinaryResponse::GetKey(response) => {
                dst.put_u32(response.flags);
                dst.put_slice(&response.key[..]);
            }
            BinaryResponse::GetKeyQuietly(response) => {
                dst.put_u32(response.flags);
                dst.put_slice(&response.key[..]);
            }

            BinaryResponse::Set(response) => {}
            BinaryResponse::Add(response) => {}
            BinaryResponse::Replace(response) => {}
        }
    }
}

impl Encoder<BinaryResponse> for MemcachedBinaryCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: BinaryResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(self.get_length(&msg));
        self.write_msg(&msg, dst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_encode_decode() {}
}
