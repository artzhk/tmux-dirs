use domain::Cmd;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::os::unix::net::{UnixListener, UnixStream};
use std::str::SplitWhitespace;

type AppState = HashMap<String, Vec<String>>;

fn handle_stream(state: &mut AppState, mut s: UnixStream) -> std::io::Result<()> {
    use std::io::prelude::*;

    fn validate(buf: &SplitWhitespace<'_>) -> bool {
        fn is_cmd_tok(l: usize, s: &str) -> bool {
            return s.len() == 1 && l >= 2;
        }

        let v: Vec<&str> = buf.clone().collect();
        if v.len() == 0 {
            return false
        }
        return (is_cmd_tok(v.len(), v[0]) && v.len() == 3 && v[2].contains("/"))
            || (is_cmd_tok(v.len(), v[0]));
    }

    let mut buf = String::new();
    s.read_to_string(&mut buf)?;

    let mut params = buf.split_whitespace();
    if !validate(&params) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Command structure is invalid",
        ));
    }

    fn get_part<'a>(p: Option<&'a str>, message: &'a str) -> io::Result<&'a str> {
        if let Some(r) = p {
            return Ok(r);
        }

        Err(io::Error::new(io::ErrorKind::InvalidData, message))
    }

    let cmd = Cmd::from_char(
        get_part(params.next(), "Command token")
            .unwrap()
            .chars()
            .next()
            .unwrap(),
    )
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid command token"))?;
    let session = get_part(params.next(), "Session part")?;

    let msg: String;
    match cmd {
        Cmd::Push => msg = pushd(state, session, get_part(params.next(), "Path part")?),
        Cmd::Pop => msg = popd(state, session),
        Cmd::Peek => msg = peekd(state, session),
        Cmd::Dirs => msg = dirs(state, session).join(" "),
    }

    s.write_all(format!("{}", msg).as_bytes())
        .map_err(|e| io::Error::new(e.kind(), format!("write response failed: {e}")))?;
    s.flush()
        .map_err(|e| io::Error::new(e.kind(), format!("flush (Write) failed: {e}")))?;
    s.shutdown(std::net::Shutdown::Write)
        .map_err(|e| io::Error::new(e.kind(), format!("shutdown(Write) failed: {e}")))?;

    Ok(())
}

fn peekd(stack: &mut AppState, session: &str) -> String {
    if let Some(v) = stack.get(session)
        && v.len() > 0
    {
        return v[v.len() - 1].clone();
    }

    "".to_string()
}

fn popd(stack: &mut AppState, session: &str) -> String {
    if let Some(r) = stack.get_mut(session)
        && let Some(msg) = r.pop()
    {
        return msg;
    }

    "".to_string()
}

fn pushd(stack: &mut AppState, session: &str, path: &str) -> String {
    let session_str = session.to_string();
    stack
        .entry(session_str)
        .and_modify(|vec| vec.push(path.to_string()))
        .or_insert_with(|| vec![path.to_string()]);

    path.to_string()
}

fn dirs(stack: &AppState, session: &str) -> Vec<String> {
    return match stack.get(session) {
        Some(v) => {
            let mut r = v.clone();
            r.reverse();
            return r;
        }
        None => vec![],
    };
}

fn main() -> std::io::Result<()> {
    use std::fs::remove_file;
    use std::path::Path;

    use signal_hook::consts::signal::{SIGINT, SIGQUIT, SIGTERM};
    use signal_hook::flag;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    let path = "/tmp/dirs.sock";
    if Path::new(&path).exists() {
        match UnixStream::connect(&path) {
            Ok(_) => {
                eprintln!("another instance is already running, shutdown...");
                //std::process::exit(0);
            }
            Err(_) => {
                remove_file(&path)?;
            }
        }
    }

    let listener = UnixListener::bind(path)
        .map_err(|e| io::Error::new(e.kind(), format!("bind({path}) failed: {e}")))?;
    listener
        .set_nonblocking(false)
        .map_err(|e| io::Error::new(e.kind(), format!("set_nonblocking failed: {e}")))?;

    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, shutdown.clone())
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("register SIGINT failed: {e}")))?;
    flag::register(SIGTERM, shutdown.clone())
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("register SIGTERM failed: {e}")))?;
    flag::register(SIGQUIT, shutdown.clone())
        .map_err(|e| io::Error::new(ErrorKind::Other, format!("register SIGQUIT failed: {e}")))?;

    let mut state: AppState = HashMap::new();

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                if let Err(err) = handle_stream(&mut state, stream) {
                    eprintln!("handle_stream error: {err}");
                    continue;
                }
            }
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                eprintln!("another instance is already running, shutdown...");
                std::process::exit(0);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
            Err(err) => {
                eprintln!("accept failed: {err}");
                break;
            }
        }
    }

    if Path::new(&path).exists() {
        if let Err(e) = remove_file(&path) {
            eprintln!("cleanup remove_file({path}) failed: {e}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> AppState {
        let mut d: HashMap<String, Vec<String>> = HashMap::new();
        let mut v: Vec<&str> = Vec::new();
        v.push("dir1");
        v.push("dir2");

        d.insert(
            "session".to_string(),
            v.iter().map(|x| x.to_string()).collect(),
        );
        return d;
    }

    #[test]
    fn popd_test() {
        let mut state: AppState = test_data();
        let res = popd(&mut state, "session");
        assert!(state
            .get(&"session".to_string())
            .is_some_and(|x| x[0] == "dir1".to_string()));
        assert_eq!(res, "dir2".to_string());
    }

    #[test]
    fn peek_test() {
        let mut state: AppState = test_data();
        let res = peekd(&mut state, "session");
        assert!(state
            .get(&"session".to_string())
            .is_some_and(|x| x[1] == "dir2".to_string()));
        assert_eq!(res, "dir2".to_string());
    }

    #[test]
    fn dirs_test() {
        let mut state: AppState = test_data();
        let d = dirs(&mut state, "session");
        let exp: Vec<String> = vec!["dir2".to_string(), "dir1".to_string()];
        assert_eq!(d, exp);
    }

    #[test]
    fn pushd_test() {
        let mut state: AppState = test_data();
        pushd(&mut state, "test", "path");
        assert!(state
            .get(&"test".to_string())
            .is_some_and(|x| x[x.len() - 1] == "path".to_string()));
    }

    #[test]
    fn integration_test() {
        let mut state: AppState = HashMap::new();
        pushd(&mut state, "test", "path");
        let pushd_res = pushd(&mut state, "test", "last_path");

        let exp: Vec<String> = vec!["last_path".to_string(), "path".to_string()];
        assert_eq!(dirs(&state, "test"), exp);

        let peekd_res = peekd(&mut state, "test");
        let popd_res = popd(&mut state, "test");
        assert_eq!("last_path", pushd_res);
        assert_eq!("last_path", peekd_res);
        assert_eq!("last_path", popd_res);

        let exp: Vec<String> = vec!["path".to_string()];
        assert_eq!(dirs(&state, "test"), exp);
    }
}
