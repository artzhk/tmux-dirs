use domain::{Cmd, CmdError, Expression};
use std::env;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

fn get_args() -> Result<Expression, std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let l = args.len();
    assert!(l >= 3, "Invalid number of arguments");

    let action = args[1].clone();
    let ar = Cmd::from_str(&action);
    return match ar {
        Ok(a) => Ok(Expression {
            cmd: a,
            session_id: if l == 4 {
                args[3].clone()
            } else {
                args[2].clone()
            },
            path: if l == 4 {
                args[2].clone()
            } else {
                "".to_string()
            },
        }),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidInput,
            CmdError::InvalidExpression.to_string(),
        )),
    };
}

fn main() {
    if let Err(e) = cli() {
        eprintln!("{}", e);
        std::process::exit(2);
    }
}

fn cli() -> std::io::Result<()> {
    use domain::is_path_free;
    use std::io::prelude::*;
    use std::os::unix::net::UnixStream;
    use std::path::{absolute, Path};

    let args = get_args()?;
    let cmd = args.cmd.to_string();

    let path_provided: bool = Path::new(&args.path).exists() && !args.path.to_string().is_empty();

    if is_path_free(&cmd) && path_provided {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "command does not accept a path",
        ));
    }

    if !is_path_free(&cmd) && !path_provided {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "path is required and must exist",
        ));
    }

    let mut path = "".to_string();
    if !is_path_free(&cmd) {
        path = String::from(absolute(&args.path)
            .map_err(|e| Error::new(ErrorKind::InvalidInput, format!("canonicalize path failed: {e}")))?
            .to_str().ok_or_else(|| Error::new(ErrorKind::InvalidInput, "path contains invalid UTF-8"))?);
    }

    let mut stream = match UnixStream::connect("/tmp/dirs.sock") {
        Ok(sock) => sock,
        Err(err) => {
            return Err(Error::new(ErrorKind::NotFound, format!("connect /tmp/dirs.sock failed: {err}")));
        }
    };

    stream.write_all(format!("{} {} {}", args.cmd.to_char(), args.session_id, path).as_bytes())
        .map_err(|e| Error::new(e.kind(), format!("write request failed: {e}")))?;
    stream.flush().map_err(|e| Error::new(e.kind(), format!("flush failed: {e}")))?;
    stream.shutdown(std::net::Shutdown::Write)
        .map_err(|e| Error::new(e.kind(), format!("shutdown(Write) failed: {e}")))?;

    let mut r = String::new();
    stream.read_to_string(&mut r)
        .map_err(|e| Error::new(e.kind(), format!("read response failed: {e}")))?;

    println!("{}", &r);

    Ok(())
}
