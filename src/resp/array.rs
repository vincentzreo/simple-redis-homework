use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{
    calc_total_length, extract_fixed_data, parse_length, RespDecode, RespEncode, RespError,
    RespFrame,
};

use super::{BUF_CAP, CRLF_LEN};

#[derive(Debug, Clone, PartialEq)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespNullArray;

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>" - "*2\r\n$3\r\nget\r\n$5\r\nhello\r\n"

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(format!("*{}\r\n", self.len()).as_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }
        Ok(RespArray::new(frames))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

// - null array: "*-1\r\n"
impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        extract_fixed_data(buf, "*-1\r\n", "Null Array")?;
        Ok(RespNullArray)
    }
    fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
        Ok(5)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RespArray {
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(s.into())
    }
}

impl From<Vec<RespFrame>> for RespArray {
    fn from(s: Vec<RespFrame>) -> Self {
        RespArray(s)
    }
}

#[cfg(test)]
mod tests {
    use crate::BulkString;

    use super::*;
    #[test]
    fn test_array_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");
        let frame = RespArray::decode(&mut buf).unwrap();
        assert_eq!(
            frame,
            RespArray::new(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into(),
                BulkString::new(b"world".to_vec()).into(),
            ])
        );
    }

    #[test]
    fn test_null_array_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");
        let frame = RespNullArray::decode(&mut buf).unwrap();
        assert_eq!(frame, RespNullArray);
    }

    #[test]
    fn test_array() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".as_bytes().to_vec()).into(),
            BulkString::new("hello".as_bytes().to_vec()).into(),
            BulkString::new("world".as_bytes().to_vec()).into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_null_array() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n");
    }
}
