use tracing::warn;

use crate::{cmd::extract_args, RespArray, RespFrame};

use super::{validate_command, CommandError, CommandExecutor, Echo, HMGet};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::Backend) -> crate::RespFrame {
        crate::SimpleString::new(self.message).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(message)) => Ok(Echo {
                message: String::from_utf8(message.0.unwrap())?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid message".to_string())),
        }
    }
}

impl CommandExecutor for HMGet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let key = self.key.clone();
        let mut ret = vec![];
        for field in self.fields {
            if let Some(value) = backend.hget(&key, &field) {
                ret.push(value);
            } else {
                ret.push(RespFrame::SimpleString(crate::SimpleString(
                    "(nil)".to_string(),
                )));
            }
        }
        RespArray::new(ret).into()
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(key)) => String::from_utf8(key.0.unwrap())?,
            _ => {
                warn!("Invalid key");
                return Err(CommandError::InvalidArgument("Invalid key".to_string()));
            }
        };
        let mut fields = vec![];
        while let Some(RespFrame::BulkString(field)) = args.next() {
            fields.push(String::from_utf8(field.0.unwrap())?);
        }
        Ok(HMGet { key, fields })
    }
}
