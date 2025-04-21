use clap::Parser;
// inspiration from: https://github.com/diwic/dbus-rs/tree/master?tab=readme-ov-file#client
use dbus::blocking::Connection;
use debouncerd::{DEBOUNCE_CMD_METHOD, DEBOUNCE_METHOD, DEST, DebounceCmdOptions, DebounceOptions};
use std::{env, process::Command, time::Duration};
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

    /// Optional flag to run the command in the background (in the daemon).
    #[arg(short, long)]
    background: bool,
}

impl Args {
    fn as_debounce_opts(&self) -> DebounceOptions {
        let id = self
            .id
            .clone()
            .unwrap_or_else(|| format!("{:016x}", xxh3_64(self.cmd.as_bytes())));

        DebounceOptions {
            timeout: Duration::from_millis(self.timeout),
            id,
        }
    }

    fn as_debounce_cmd_opts(&self) -> DebounceCmdOptions {
        DebounceCmdOptions {
            timeout: Duration::from_millis(self.timeout),
            cmd: self.cmd.clone(),
            id: self
                .id
                .clone()
                .unwrap_or_else(|| format!("{:016x}", xxh3_64(self.cmd.as_bytes()))),
            pwd: self.pwd.clone(),
        }
    }
}

// dbus-send --print-reply --dest=com.example.dbustest / com.example.dbustest.Debounce string:MyName uint64:2000 string:$(pwd) string:'ls -al'

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let opts = args.as_debounce_opts();
    println!("{:?}", opts);

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(DEST, "/", Duration::from_millis(5000));

    let result: (bool, u64) = if args.background {
        let opts = args.as_debounce_cmd_opts();
        proxy.method_call(DEST, DEBOUNCE_CMD_METHOD, opts.into_tuple())?
    } else {
        let opts = args.as_debounce_opts();
        proxy.method_call(DEST, DEBOUNCE_METHOD, opts.into_tuple())?
    };

    if result.0 {
        let s = shell_words::split(&args.cmd)?;
        Command::new(&s[0])
            .current_dir(args.pwd)
            .args(&s[1..])
            .spawn()?
            .wait()?;
    } else {
        println!("timeout: {}ms", result.1)
    }

    Ok(())
}

fn default_pwd() -> String {
    env::var("PWD").unwrap_or("".into())
}
