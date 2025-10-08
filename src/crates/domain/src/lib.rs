use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Cmd {
    Push,
    Pop,
    Peek,
    Dirs,
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Expression {
    pub cmd: Cmd,
    pub session_id: String,
    pub path: String,
}

#[derive(Debug)]
pub enum CmdError {
    InvalidExpression,
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Cmd {
    pub fn from_char(c: char) -> Result<Cmd, CmdError> {
        match c {
            '\x01' => Ok(Cmd::Push),
            '\x02' => Ok(Cmd::Pop),
            '\x03' => Ok(Cmd::Peek),
            '\x04' => Ok(Cmd::Dirs),
            _ => Err(CmdError::InvalidExpression),
            //_ => Err(Error::new(ErrorKind::InvalidInput, CmdError::InvalidExpression.to_string())),
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            Cmd::Push => '\x01',
            Cmd::Pop => '\x02',
            Cmd::Peek => '\x03',
            Cmd::Dirs => '\x04',
        }
    }
}

impl FromStr for Cmd {
    type Err = ();

    fn from_str(input: &str) -> Result<Cmd, Self::Err> {
        match input {
            "pushd" => Ok(Cmd::Push),
            "popd" => Ok(Cmd::Pop),
            "peekd" => Ok(Cmd::Peek),
            "dirs" => Ok(Cmd::Dirs),
            _ => Err(()),
        }
    }
}

pub fn is_path_free(cmd: &String) -> bool {
    return *cmd != Cmd::Push.to_string(); 
}

