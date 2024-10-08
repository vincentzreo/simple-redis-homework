use std::ops::Deref;

use bytes::{Buf, BytesMut};

use crate::{parse_length, RespDecode, RespEncode, RespError};

use super::CRLF_LEN;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkString(pub(crate) Option<Vec<u8>>);

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct RespNullBulkString;

// - bulk string: "$<length>\r\n<data>\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        match self.0 {
            None => b"$-1\r\n".to_vec(),
            Some(data) => {
                let mut buf = Vec::with_capacity(data.len() + 16);
                buf.extend_from_slice(format!("${}\r\n", data.len()).as_bytes());
                buf.extend_from_slice(&data);
                buf.extend_from_slice(b"\r\n");
                buf
            }
        }
    }
}

// // - null bulk string: "$-1\r\n"
// impl RespEncode for RespNullBulkString {
//     fn encode(self) -> Vec<u8> {
//         b"$-1\r\n".to_vec()
//     }
// }

// impl RespDecode for RespNullBulkString {
//     const PREFIX: &'static str = "$";
//     fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
//         extract_fixed_data(buf, "$-1\r\n", "Null Bulk String")?;
//         Ok(RespNullBulkString)
//     }
//     fn expect_length(_buf: &[u8]) -> Result<usize, RespError> {
//         Ok(5)
//     }
// }

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::new(data[..len].to_vec()))
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl Deref for BulkString {
    type Target = Option<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(Some(s.into()))
    }
    pub fn new_null() -> Self {
        BulkString(None)
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(Some(s.as_bytes().to_vec()))
    }
}

impl From<&[u8]> for BulkString {
    fn from(value: &[u8]) -> Self {
        BulkString(Some(value.to_vec()))
    }
}

impl<const N: usize> From<&[u8; N]> for BulkString {
    fn from(value: &[u8; N]) -> Self {
        BulkString(Some(value.to_vec()))
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_bulk_string_decode() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = BulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, BulkString::new(b"hello".to_vec()));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf).unwrap();
        assert_eq!(frame, BulkString::new(b"hello".to_vec()));
    }

    // #[test]
    // fn test_null_bulk_string_decode() {
    //     let mut buf = BytesMut::new();
    //     buf.extend_from_slice(b"$-1\r\n");
    //     let frame = RespNullBulkString::decode(&mut buf).unwrap();
    //     assert_eq!(frame, RespNullBulkString);
    // }

    #[test]
    fn test_bulk_string() {
        let frame: RespFrame = BulkString::new("hello".as_bytes().to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");

        let frame: RespFrame = BulkString::new_null().into();
        assert_eq!(frame.encode(), b"$-1\r\n");
    }

    // #[test]
    // fn test_null_bulk_string() {
    //     let frame: RespFrame = RespNullBulkString.into();
    //     assert_eq!(frame.encode(), b"$-1\r\n");
    // }
}
