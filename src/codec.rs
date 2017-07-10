use tokio_io::codec::{Decoder, Encoder};
use bytes::BytesMut;
use std::io;
use insteon_structs::*;

pub struct LineCodec;


impl Decoder for LineCodec {
    type Item = InsteonMsg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        const COMMAND_START :u8 = 0x02;
        match src.iter().position(|x| *x == COMMAND_START ) {
            Some(idx) => src.split_to(idx + 1),
            None => return Ok(None)
        };

        match InsteonMsg::new(&src) {
            Some((msg, msg_size)) => {
                src.split_to(msg_size);
                Ok(Some(msg))
            },
            None => Ok(None)
        }

    }
}

impl Encoder for LineCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, _item: Self::Item, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        _dst.extend_from_slice(&_item);
        Ok(())
    }
}