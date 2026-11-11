//! A newer afk release (ADR 0022): the check behind the notice in the menu
//! and in `--list`, and the binary step of `--update`, which installs the
//! release over this binary through dist's install receipt.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axoupdater::{AxoUpdater, ReleaseSource, ReleaseSourceType, Version};

/// The dist app: the receipt's name and the release assets' prefix.
const APP: &str = "agentic-field-kit";
const OWNER: &str = "DNRN";
/// How long a check's answer holds, in seconds.
const FRESH: u64 = 24 * 60 * 60;
/// Per request in the background check, so a dead network can't hold it for minutes.
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);
/// Per request in the binary step: the release list and the install script.
const UPDATE_TIMEOUT: Duration = Duration::from_secs(30);

/// Set to anything but empty or `0`: afk makes no network request at all.
pub const OFF_VAR: &str = "AFK_NO_UPDATE_CHECK";
/// Set on the run that follows an update, so that run doesn't update again.
pub const UPDATED_VAR: &str = "AFK_UPDATED";
/// How to update an afk that the install script didn't install.
pub const MANUAL: &str = "cargo install --locked --force --git https://github.com/DNRN/agentic-field-kit agentic-field-kit, or rerun the install script from the README";

/// `AFK_NO_UPDATE_CHECK` is set.
pub fn off() -> bool {
    env::var_os(OFF_VAR).is_some_and(|v| !v.is_empty() && v != "0")
}

/// This run follows an update.
pub fn updated() -> bool {
    env::var_os(UPDATED_VAR).is_some()
}

/// `latest`, when it is a newer version than `current`.
pub fn newer(current: &str, latest: &str) -> Option<String> {
    let (Ok(current), Ok(latest)) = (Version::parse(current), Version::parse(latest)) else {
        return None;
    };
    (latest > current).then(|| latest.to_string())
}

/// The latest release a check saw, and when (seconds since the epoch). On
/// disk: `0.6.0 1762855200`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub latest: String,
    pub at: u64,
}

impl Seen {
    pub fn parse(text: &str) -> Option<Seen> {
        let mut words = text.split_whitespace();
        let latest = words.next()?;
        let at = words.next()?.parse().ok()?;
        Version::parse(latest).ok()?;
        Some(Seen {
            latest: latest.to_owned(),
            at,
        })
    }

    pub fn text(&self) -> String {
        format!("{} {}\n", self.latest, self.at)
    }

    /// Seen less than a day before `now`. A time after `now` is stale: the
    /// clock was wrong at one of the two.
    pub fn fresh(&self, now: u64) -> bool {
        now.checked_sub(self.at).is_some_and(|age| age < FRESH)
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Where the check keeps what it saw, in the user's cache folder.
pub fn cache_file(home: &Path) -> PathBuf {
    let dir = if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .filter(|d| !d.is_empty())
            .map_or_else(|| home.join("AppData").join("Local"), PathBuf::from)
    } else if cfg!(target_os = "macos") {
        home.join("Library").join("Caches")
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|d| d.is_absolute())
            .unwrap_or_else(|| home.join(".cache"))
    };
    dir.join(APP).join("latest-release")
}

fn remember(file: &Path, latest: &str, at: u64) {
    if let Some(dir) = file.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let seen = Seen {
        latest: latest.to_owned(),
        at,
    };
    let _ = fs::write(file, seen.text());
}

/// What a check knows: a newer version now, and maybe a better answer later.
pub struct Check {
    pub newer: Option<String>,
    /// With a stale or missing cache: the background query's answer (the
    /// newer version, or `None` when this one is the latest). It disconnects
    /// without an answer when the query fails.
    pub later: Option<Receiver<Option<String>>>,
}

impl Check {
    /// The answer, waiting up to `limit` for the background query.
    pub fn wait(self, limit: Duration) -> Option<String> {
        match self.later {
            Some(later) => later.recv_timeout(limit).unwrap_or(self.newer),
            None => self.newer,
        }
    }
}

/// Starts a check for the running `current` version. A fresh cache answers
/// at once; otherwise GitHub is asked in the background, and until it
/// answers, the stale cache does. Errors are silent, and a failed query
/// counts as a check too, so an unreachable GitHub is asked once a day,
/// not on every run.
pub fn check(current: &str, home: &Path) -> Check {
    check_with(current, &cache_file(home), now(), |current| {
        latest(current, CHECK_TIMEOUT)
    })
}

fn check_with(
    current: &str,
    file: &Path,
    now: u64,
    ask: impl FnOnce(&str) -> Result<String, String> + Send + 'static,
) -> Check {
    let seen = fs::read_to_string(file)
        .ok()
        .and_then(|text| Seen::parse(&text));
    let newer_seen = seen.as_ref().and_then(|s| newer(current, &s.latest));
    if seen.as_ref().is_some_and(|s| s.fresh(now)) {
        return Check {
            newer: newer_seen,
            later: None,
        };
    }
    // Record the attempt first, with the last version seen (or, with none,
    // 0.0.0, which is never newer): a query that fails, or that outlives the
    // process, still counts as the day's check.
    let last = seen.map_or_else(|| "0.0.0".to_owned(), |s| s.latest);
    remember(file, &last, now);
    let (tx, rx) = mpsc::channel();
    let (current, file) = (current.to_owned(), file.to_owned());
    thread::spawn(move || {
        if let Ok(latest) = ask(&current) {
            remember(&file, &latest, now);
            let _ = tx.send(newer(&current, &latest));
        }
    });
    Check {
        newer: newer_seen,
        later: Some(rx),
    }
}

/// A newer version from the cache alone, however old: for runs that make
/// no network request.
pub fn cached(current: &str, home: &Path) -> Option<String> {
    let seen = Seen::parse(&fs::read_to_string(cache_file(home)).ok()?)?;
    newer(current, &seen.latest)
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}

/// Points `updater` at this repository's GitHub releases, with `current` as
/// the running version (a receipt's may be older) and a limit per request.
fn configure(updater: &mut AxoUpdater, current: &str, timeout: Duration) -> Result<(), String> {
    let version = Version::parse(current).map_err(|e| format!("version {current}: {e}"))?;
    let client = reqwest::Client::builder()
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())?;
    updater
        .set_client(client)
        .set_release_source(ReleaseSource {
            release_type: ReleaseSourceType::GitHub,
            owner: OWNER.to_owned(),
            name: APP.to_owned(),
            app_name: APP.to_owned(),
        })
        .set_current_version(version)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn query(updater: &mut AxoUpdater) -> Result<String, String> {
    runtime()?
        .block_on(async {
            updater
                .query_new_version()
                .await
                .map(|v| v.map(ToString::to_string))
        })
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no release found".to_owned())
}

/// The latest release's version, from GitHub.
pub fn latest(current: &str, timeout: Duration) -> Result<String, String> {
    let mut updater = AxoUpdater::new_for(APP);
    configure(&mut updater, current, timeout)?;
    query(&mut updater)
}

/// What `--update` can do about the binary.
pub enum Binary {
    /// This is the latest release.
    Current,
    /// A newer release exists, but the install script didn't install this
    /// binary (`cargo install`, a build): afk leaves it alone.
    Manual(String),
    /// A newer release exists, and afk can install it over itself.
    Replace(Replace),
}

pub struct Replace {
    pub latest: String,
    updater: Box<AxoUpdater>,
}

impl Replace {
    /// Runs the release's install script into the receipt's folder. Its
    /// output goes to the terminal.
    pub fn apply(mut self) -> Result<(), String> {
        match runtime()?.block_on(self.updater.run()) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err("the install receipt no longer matches this afk".to_owned()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Asks GitHub for the latest release, and the install receipt whether it
/// describes this binary. Remembers the answer for the check.
pub fn binary(current: &str, home: &Path) -> Result<Binary, String> {
    let mut updater = AxoUpdater::new_for(APP);
    let ours = updater.load_receipt().is_ok()
        && updater
            .check_receipt_is_for_this_executable()
            .unwrap_or(false);
    if !ours {
        updater = AxoUpdater::new_for(APP);
    }
    configure(&mut updater, current, UPDATE_TIMEOUT)?;
    let latest = query(&mut updater)?;
    remember(&cache_file(home), &latest, now());
    Ok(match newer(current, &latest) {
        None => Binary::Current,
        Some(latest) if ours => Binary::Replace(Replace {
            latest,
            updater: Box::new(updater),
        }),
        Some(latest) => Binary::Manual(latest),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_higher_version_is_newer() {
        assert_eq!(newer("0.5.0", "0.6.0").as_deref(), Some("0.6.0"));
        assert_eq!(newer("0.5.0", "0.10.0").as_deref(), Some("0.10.0"));
        assert_eq!(newer("0.5.0", "0.5.0"), None);
        assert_eq!(newer("0.6.0", "0.5.1"), None);
        assert_eq!(newer("0.6.0-rc.1", "0.6.0").as_deref(), Some("0.6.0"));
        assert_eq!(newer("", "0.6.0"), None);
        assert_eq!(newer("0.5.0", "garbage"), None);
    }

    #[test]
    fn the_cache_reads_back_what_it_wrote_and_rejects_the_rest() {
        let seen = Seen {
            latest: "0.6.0".to_owned(),
            at: 1_762_855_200,
        };
        assert_eq!(Seen::parse(&seen.text()), Some(seen));
        assert_eq!(Seen::parse(""), None);
        assert_eq!(Seen::parse("0.6.0"), None);
        assert_eq!(Seen::parse("0.6.0 yesterday"), None);
        assert_eq!(Seen::parse("latest 1762855200"), None);
    }

    #[test]
    fn an_answer_holds_for_a_day() {
        let seen = Seen {
            latest: "0.6.0".to_owned(),
            at: 1_000_000,
        };
        assert!(seen.fresh(1_000_000));
        assert!(seen.fresh(1_000_000 + FRESH - 1));
        assert!(!seen.fresh(1_000_000 + FRESH));
        assert!(!seen.fresh(999_999), "a check from the future is stale");
    }

    fn cache(dir: &Path, text: &str) -> PathBuf {
        let file = dir.join("latest-release");
        fs::write(&file, text).unwrap();
        file
    }

    #[test]
    fn a_fresh_cache_answers_without_asking() {
        let dir = tempfile::tempdir().unwrap();
        let file = cache(dir.path(), "0.6.0 1000\n");
        let check = check_with("0.5.0", &file, 2000, |_| panic!("asked GitHub"));
        assert_eq!(check.newer.as_deref(), Some("0.6.0"));
        assert!(check.later.is_none());
        let check = check_with("0.6.0", &file, 2000, |_| panic!("asked GitHub"));
        assert_eq!(check.newer, None);
    }

    #[test]
    fn a_stale_cache_answers_until_github_does_and_is_rewritten() {
        let dir = tempfile::tempdir().unwrap();
        let file = cache(dir.path(), "0.6.0 1000\n");
        let check = check_with("0.5.0", &file, 1000 + FRESH, |_| Ok("0.7.0".to_owned()));
        assert_eq!(check.newer.as_deref(), Some("0.6.0"));
        assert_eq!(check.wait(Duration::from_secs(5)).as_deref(), Some("0.7.0"));
        let seen = Seen::parse(&fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(seen.latest, "0.7.0");
    }

    #[test]
    fn a_failed_query_keeps_what_the_cache_said_and_waits_a_day() {
        let dir = tempfile::tempdir().unwrap();
        let file = cache(dir.path(), "0.6.0 1000\n");
        let now = 1000 + FRESH;
        let check = check_with("0.5.0", &file, now, |_| Err("offline".to_owned()));
        assert_eq!(check.wait(Duration::from_secs(5)).as_deref(), Some("0.6.0"));
        assert_eq!(fs::read_to_string(&file).unwrap(), format!("0.6.0 {now}\n"));
        let check = check_with("0.5.0", &file, now + 1, |_| panic!("asked again"));
        assert!(check.later.is_none(), "asked again");
        assert_eq!(check.newer.as_deref(), Some("0.6.0"));
    }

    #[test]
    fn a_failed_first_query_shows_nothing_and_waits_a_day() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("missing").join("latest-release");
        let check = check_with("0.5.0", &file, 5000, |_| Err("offline".to_owned()));
        assert_eq!(check.wait(Duration::from_secs(5)), None);
        assert_eq!(fs::read_to_string(&file).unwrap(), "0.0.0 5000\n");
        let check = check_with("0.5.0", &file, 5001, |_| panic!("asked again"));
        assert!(check.later.is_none(), "asked again");
        assert_eq!(check.newer, None);
    }

    #[test]
    fn up_to_date_after_the_query_clears_the_notice() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("missing").join("latest-release");
        let check = check_with("0.7.0", &file, 5000, |_| Ok("0.7.0".to_owned()));
        assert_eq!(check.wait(Duration::from_secs(5)), None);
        assert!(file.is_file(), "the answer is cached");
    }
}
