// inspiration from: https://github.com/diwic/dbus-rs/blob/master/dbus-crossroads/examples/server_cr.rs
use dbus::{MethodErr, blocking::Connection};
use dbus_crossroads::{Context, Crossroads};
use debouncerd::{MAX_ENTRIES, MAX_TIMEOUT_MS};
use std::{
    collections::HashMap,
    error::Error,
    process::Command,
    time::{Duration, Instant},
};

/// The maximum number of items in daemon memory before trigerring a cleanup.
pub const GC_ITEMS: usize = 1000; // 1_000_000;

// This is our "Hello" object that we are going to store inside the crossroads instance.
// TODO: double check this is thread safe, since this is a "bus" we can think only 1 event is consumed everytime?
#[derive(Default)]
struct Debouncer {
    // TODO: max time for debounce so we can cleanup this map and avoid memory leak?
    timers: HashMap<String, Instant>,
}

// ➜  / dbus-send --print-reply --dest=com.example.dbustest /hello com.example.dbustest.Hello string:MyName uint64:2000 string:$(pwd) string:'ls -al'

#[derive(thiserror::Error, Debug)]
enum TryRunError {
    #[error("Too many entries (max: {})", MAX_ENTRIES)]
    TooManyEntries,
    #[error("Timeout is too long (max: {})", MAX_TIMEOUT_MS)]
    TimeoutTooLong,
}

impl Debouncer {
    #[allow(clippy::zombie_processes)]
    fn run(&mut self, id: &str, pwd: &str, cmd: &str) {
        println!("exec {}", cmd);
        // TODO: better error handling here
        let s = shell_words::split(cmd).expect("parsing");
        // TODO: put the result into logger deamon?
        // TODO: or at least spawn it in a thread and follow exec but do not block the "main" thread.
        Command::new(&s[0])
            .current_dir(pwd)
            .args(&s[1..])
            .spawn()
            .expect("oups");
        self.timers.insert(id.into(), Instant::now());
    }

    fn try_run(
        &mut self,
        id: &str,
        timeout: Duration,
        pwd: &str,
        cmd: &str,
    ) -> Result<Option<Duration>, TryRunError> {
        if self.timers.len() > GC_ITEMS {
            let expired_ids: Vec<_> = self
                .timers
                .iter()
                .filter_map(|(id, timer)| {
                    if timer.elapsed() > Duration::from_secs(MAX_TIMEOUT_MS) {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect();

            for id in expired_ids {
                self.timers.remove(&id);
            }
        }

        if self.timers.len() > MAX_ENTRIES {
            return Err(TryRunError::TooManyEntries);
        }
        if timeout > Duration::from_millis(MAX_TIMEOUT_MS) {
            return Err(TryRunError::TimeoutTooLong);
        }
        let Some(timer) = self.timers.get(id) else {
            self.run(id, pwd, cmd);
            return Ok(None);
        };

        let elapsed = timer.elapsed();
        if elapsed < timeout {
            return Ok(Some(timeout - elapsed));
        }

        self.run(id, pwd, cmd);
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let c = Connection::new_session()?;
    c.request_name("com.example.dbustest", false, true, false)?;

    let mut cr = Crossroads::new();

    // TODO: add the "cron" method
    // TODO: add the "list" method -> returning state of debounces/cron/etc
    let iface_token = cr.register("com.example.dbustest", |b| {
        b.method(
            "Debounce",
            ("id", "duration_ms", "pwd", "cmd"),
            ("executed", "timeout"),
            move |_: &mut Context,
                  debouncer: &mut Debouncer,
                  (id, duration_ms, pwd, cmd): (String, u64, String, String)| {
                match debouncer.try_run(&id, Duration::from_millis(duration_ms), &pwd, &cmd) {
                    Ok(res) => {
                        let executed = res.is_none();
                        let timeout: u64 = (res.unwrap_or(Duration::ZERO).as_millis())
                            .try_into()
                            .expect("timeout should fit into a u64");

                        println!("{} - executed: {} - timeout: {}", cmd, executed, timeout);

                        Ok((executed, timeout))
                    }
                    Err(e) => {
                        eprintln!("try_run error: {}", e);
                        Err(MethodErr::failed(&e.to_string()))
                    }
                }
            },
        );
    });

    cr.insert("/", &[iface_token], Debouncer::default());
    cr.serve(&c)?;
    unreachable!()
}

// dbus-send --session --type=method_call --print-reply \
// --dest=com.example.dbustest \
// / \
// org.freedesktop.DBus.Introspectable.Introspect
