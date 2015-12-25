use std::io::Read;
use std::io::Write;
use std::str::FromStr;

use protocol::cmd::Cmd;
use protocol::cmd::Get;
use protocol::cmd::Resp;
use protocol::cmd::Set;

use super::errors::TcpTransportError;
use super::typedefs::TcpTransportResult;


pub struct TcpTransport<T> {
    stream: T,
    key_maxlen: u64,
}

impl<T: Read + Write> TcpTransport<T> {
    pub fn new(mut stream: T) -> TcpTransport<T> {
        TcpTransport {
            stream: stream,
            key_maxlen: 250, // memcached standard
        }
    }

    pub fn with_key_maxlen(&mut self,
                           key_maxlen: u64)
                           -> &mut TcpTransport<T> {
        self.key_maxlen = key_maxlen;
        self
    }


    pub fn get_max_line_len(&self) -> usize {
        // This needs to be the length of the longest command line, not
        // including data values for which the length is given upfront
        self.key_maxlen as usize + 100
    }

    // Basic bytes manipulation

    pub fn as_string(&self, bytes: Vec<u8>) -> TcpTransportResult<String> {
        match String::from_utf8(bytes) {
            Ok(string) => Ok(string),
            Err(_) => Err(TcpTransportError::Utf8Error),
        }
    }

    pub fn as_number<N: FromStr>(&self,
                                 bytes: Vec<u8>)
                                 -> TcpTransportResult<N> {
        let string = try!(self.as_string(bytes));
        match string.parse::<N>() {
            Ok(num) => Ok(num),
            Err(_) => Err(TcpTransportError::NumberParseError),
        }
    }

    pub fn read_byte(&mut self) -> TcpTransportResult<u8> {
        let mut bytes = [0; 1];

        match self.stream.read(&mut bytes) {
            Ok(1) => Ok(bytes[0]),
            _ => Err(TcpTransportError::SocketReadError),
        }
    }

    pub fn read_bytes(&mut self, len: u64) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![];

        for _ in 0..len {
            let byte = try!(self.read_byte());
            bytes.push(byte);
        }

        Ok(bytes)
    }

    pub fn read_line(&mut self, maxlen: usize) -> TcpTransportResult<Vec<u8>> {
        let mut bytes = vec![];
        let mut found_line_end = false;

        for _ in 0..maxlen {
            let byte = try!(self.read_byte());
            bytes.push(byte);

            // Look for \r\n
            if bytes.ends_with(&[13, 10]) {
                found_line_end = true;
                break;
            }
        }

        if found_line_end {
            // Pop off \r\n
            bytes.pop();
            bytes.pop();
            Ok(bytes)
        } else {
            Err(TcpTransportError::LineReadError)
        }
    }

    pub fn parse_word(&self,
                      bytes: Vec<u8>)
                      -> TcpTransportResult<(Vec<u8>, Vec<u8>)> {
        let mut space_idx = -1;

        for i in 0..bytes.len() {
            // We're looking for a space
            if bytes[i] == 32 {
                space_idx = i;
                break;
            }
        }

        if space_idx as i64 > -1 {
            let mut word = vec![];
            let mut rest = vec![];

            // TODO figure out how to return a modified vector instead of
            // copying the whole rest of it
            for i in 0..bytes.len() {
                let byte = bytes[i];
                if i < space_idx {
                    word.push(byte);
                } else {
                    rest.push(byte);
                }
            }

            Ok((word, rest))

        } else {
            // If we've reached the end of the buffer without seeing a space
            // that makes the whole buffer a word
            Ok((bytes, vec![]))
        }
    }

    // Parse individual commands

    pub fn parse_cmd_get(&self, mut rest: Vec<u8>) -> TcpTransportResult<Cmd> {
        rest.remove(0); // remove leading space XXX errors
        let (key, rest) = try!(self.parse_word(rest));

        // We expect to find the end of the line now
        if rest.is_empty() {
            let key_str = try!(self.as_string(key));
            Ok(Cmd::Get(Get { key: key_str }))
        } else {
            Err(TcpTransportError::CommandParseError)
        }
    }

    pub fn parse_cmd_set(&mut self,
                         mut rest: Vec<u8>)
                         -> TcpTransportResult<Cmd> {
        rest.remove(0); // remove leading space XXX errors
        let (key, mut rest) = try!(self.parse_word(rest));

        rest.remove(0); // remove leading space XXX errors
        let (flags, mut rest) = try!(self.parse_word(rest));

        rest.remove(0); // remove leading space XXX errors
        let (exptime, mut rest) = try!(self.parse_word(rest));

        rest.remove(0); // remove leading space XXX errors
        let (bytelen, rest) = try!(self.parse_word(rest));

        let key_str = try!(self.as_string(key));
        let flags_num = try!(self.as_number::<u16>(flags));
        let exptime_num = try!(self.as_number::<u32>(exptime));
        let bytelen_num = try!(self.as_number::<u64>(bytelen));

        // We know the byte length, so now read the value
        let value = try!(self.read_bytes(bytelen_num));

        // Read the line termination marker
        let line_len = self.get_max_line_len();
        let rest = try!(self.read_line(line_len));

        // We got all the values we expected and there is nothing left
        if rest.is_empty() {
            return Ok(Cmd::Set(Set {
                key: key_str,
                exptime: exptime_num,
                data: value,
            }));
        }

        Err(TcpTransportError::CommandParseError)
    }

    // High level functions

    pub fn read_cmd(&mut self) -> TcpTransportResult<Cmd> {
        let line_len = self.get_max_line_len();

        let fst_line = try!(self.read_line(line_len));
        let (keyword, rest) = try!(self.parse_word(fst_line));
        let keyword_str = try!(self.as_string(keyword));

        if keyword_str == "get" {
            return self.parse_cmd_get(rest);
        } else if keyword_str == "set" {
            return self.parse_cmd_set(rest);
        } else if keyword_str == "stats" {
            return Ok(Cmd::Stats);
        }

        Err(TcpTransportError::InvalidCmd)
    }

    pub fn write_resp(&mut self, resp: &Resp) -> TcpTransportResult<()> {
        Ok(())
    }
}
