use std::time::Duration;

// TODO: use clap to override this in the daemon
pub const MAX_TIMEOUT_MS: u64 = 1000 * 60 * 60 * 24; // 1 day

// TODO: use clap to override this in the daemon
pub const MAX_ENTRIES: usize = 1_000_000;

pub const DEST: &str = "com.github.fabienjuif.debouncerd";

pub const DEBOUNCE_CMD_METHOD: &str = "DebounceCmd";

#[derive(Debug)]
pub struct DebounceCmdOptions {
    pub timeout: Duration,
    pub cmd: String,
    pub id: String,
    pub pwd: String,
}

impl DebounceCmdOptions {
    pub fn input_args() -> (&'static str, &'static str, &'static str, &'static str) {
        ("id", "timeout", "pwd", "cmd")
    }

    pub fn into_tuple(self) -> (String, u64, String, String) {
        (
            self.id,
            self.timeout
                .as_millis()
                .try_into()
                .expect("timeout should fit into u64"),
            self.pwd,
            self.cmd,
        )
    }

    pub fn from_tuple((id, timeout, pwd, cmd): (String, u64, String, String)) -> Self {
        Self {
            timeout: Duration::from_millis(timeout),
            cmd,
            id,
            pwd,
        }
    }
}
