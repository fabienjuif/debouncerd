use clap::Parser;
// inspiration from: https://github.com/diwic/dbus-rs/tree/master?tab=readme-ov-file#client
use dbus::blocking::Connection;
use debouncerd::{DEBOUNCE_METHOD, DEST, DebounceOptions};
use std::{env, time::Duration};
use xxhash_rust::xxh3::xxh3_64;

// TODO: share some constant&struct in src/lib with daemon

// TODO: error if timeout < 1ms || timeout > 24h
// TODO: return error from server (and then server have to return error too)
// TODO: for now we do not return stdout or stderr from executed command
//       my feelin is that his is too heavy for a bus, we may want to use a
//       tmp filedescriptor maybe?

/// A debounce wrapper that runs a command with a timeout and optional settings.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Debounce timeout in milliseconds
    #[arg(index = 1)]
    timeout: u64,

    /// Command to execute
    #[arg(index = 2)]
    cmd: String,

    /// Optional identifier for the debounce group (useful for distinguishing runs)
    #[arg(long)]
    id: Option<String>,

    /// Optional present working directory to run the command from
    #[arg(long, default_value_t = default_pwd())]
    pwd: String,
}

impl Args {
    fn with_defaults(self) -> DebounceOptions {
        let id = self
            .id
            .unwrap_or_else(|| format!("{:016x}", xxh3_64(self.cmd.as_bytes())));

        DebounceOptions {
            timeout: Duration::from_millis(self.timeout),
            cmd: self.cmd,
            id,
            pwd: self.pwd,
        }
    }
}

// dbus-send --print-reply --dest=com.example.dbustest / com.example.dbustest.Debounce string:MyName uint64:2000 string:$(pwd) string:'ls -al'

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Args::parse().with_defaults();
    println!("{:?}", options);

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(DEST, "/", Duration::from_millis(5000));

    let (executed, timeout): (bool, u64) =
        proxy.method_call(DEST, DEBOUNCE_METHOD, options.into_tuple())?;

    if executed {
        println!("executed!")
    } else {
        println!("timeout: {}ms", timeout)
    }

    Ok(())
}

fn default_pwd() -> String {
    env::var("PWD").unwrap_or("".into())
}
