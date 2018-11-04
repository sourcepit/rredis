use std::error::Error;
use std::io::Read;
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

#[derive(PartialEq, Debug)]
enum RedisType {
    SimpleString,
    Error,
    Integer,
    BulkString,
    Array,
}

fn read_type(r: &mut Read) -> Result<RedisType> {
    let mut buf: [u8; 1] = [0];
    let t: u8 = match r.read_exact(&mut buf) {
        Ok(()) => buf[0],
        Err(e) => return Err(Box::from(e)),
    };
    let redis_type: RedisType = match t {
        b'+' => RedisType::SimpleString,
        b'-' => RedisType::Error,
        b':' => RedisType::Integer,
        b'$' => RedisType::BulkString,
        b'*' => RedisType::Array,
        _ => {
            return Err(Box::from(format!(
                "Unknown RESP data type idenifier '{}'",
                t
            )))
        }
    };
    Ok(redis_type)
}

fn read_to_string(r: &mut Read) -> Result<String> {
    let mut buf = String::new();
    match r.read_to_string(&mut buf) {
        Ok(_) => Ok(buf),
        Err(e) => Err(Box::new(e)),
    }
}

fn read_integer(r: &mut Read) -> Result<i32> {
    let mut reader = SimpleStringReader::new(r);
    match read_to_string(&mut reader) {
        Ok(s) => match s.parse::<i32>() {
            Ok(i) => Ok(i),
            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(e),
    }
}

struct SimpleStringReader<'a> {
    inner: &'a mut Read,
    done: bool,
}

impl<'a> SimpleStringReader<'a> {
    fn new(r: &mut Read) -> SimpleStringReader {
        SimpleStringReader {
            inner: r,
            done: false,
        }
    }
}

impl<'a> Read for SimpleStringReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        let mut result: std::result::Result<usize, std::io::Error>;
        if self.done {
            result = Ok(0);
        } else {
            result = Ok(buf.len());
            let mut i: usize = 0;
            while i < buf.len() {
                let mut next: [u8; 1] = [0];
                let next: u8 = match self.inner.read_exact(&mut next) {
                    Ok(()) => next[0],
                    Err(e) => return Err(e),
                };
                if next == b'\r' {
                    let mut next: [u8; 1] = [0];
                    let next: u8 = match self.inner.read_exact(&mut next) {
                        Ok(()) => next[0],
                        Err(e) => return Err(e),
                    };
                    if next == b'\n' {
                        result = Ok(i);
                    } else {
                        let err = std::io::Error::from(std::io::ErrorKind::InvalidData);
                        result = Err(err);
                    }
                    self.done = true;
                    break;
                } else {
                    buf[i] = next;
                    i += 1;
                }
            }
        }
        result
    }
}

struct BulkStringReader<'a> {
    length: i32,
    position: u32,
    inner: &'a mut Read,
}

impl<'a> Read for SimpleStringReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {





        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_integer() {
        let mut bytes = "".as_bytes();
        assert!(read_integer(&mut bytes).is_err());

        let mut bytes = "\r\n".as_bytes();
        assert!(read_integer(&mut bytes).is_err());

        let mut bytes = "0\r\n".as_bytes();
        assert_eq!(0, read_integer(&mut bytes).unwrap());

        let mut bytes = "-0\r\n".as_bytes();
        assert_eq!(0, read_integer(&mut bytes).unwrap());

        let mut bytes = "-1\r\n".as_bytes();
        assert_eq!(-1, read_integer(&mut bytes).unwrap());
    }

    #[test]
    fn test_simple_string() {
        let mut bytes = "\r".as_bytes();
        let mut reader = SimpleStringReader::new(&mut bytes);
        assert!(read_to_string(&mut reader).is_err());

        let mut bytes = "Foo\rBar\r\n".as_bytes();
        let mut reader = SimpleStringReader::new(&mut bytes);
        assert!(read_to_string(&mut reader).is_err());

        let mut bytes = "\r\n".as_bytes();
        let mut reader = SimpleStringReader::new(&mut bytes);
        assert_eq!("", read_to_string(&mut reader).unwrap());

        let mut bytes = "OK\r\n".as_bytes();
        let mut reader = SimpleStringReader::new(&mut bytes);
        assert_eq!("OK", read_to_string(&mut reader).unwrap());

        let mut bytes = "Foo\r\nBar".as_bytes();
        let mut reader = SimpleStringReader::new(&mut bytes);
        assert_eq!("Foo", read_to_string(&mut reader).unwrap());
        assert_eq!("", read_to_string(&mut reader).unwrap());
    }

    #[test]
    fn test_read_type() {
        let mut bytes = "".as_bytes();
        assert!(read_type(&mut bytes).is_err());

        let mut bytes = "?".as_bytes();
        assert!(read_type(&mut bytes).is_err());

        let mut bytes = "+".as_bytes();
        assert_eq!(RedisType::SimpleString, read_type(&mut bytes).unwrap());

        let mut bytes = "-".as_bytes();
        assert_eq!(RedisType::Error, read_type(&mut bytes).unwrap());

        let mut bytes = ":".as_bytes();
        assert_eq!(RedisType::Integer, read_type(&mut bytes).unwrap());

        let mut bytes = "$".as_bytes();
        assert_eq!(RedisType::BulkString, read_type(&mut bytes).unwrap());

        let mut bytes = "*".as_bytes();
        assert_eq!(RedisType::Array, read_type(&mut bytes).unwrap());
    }
}
