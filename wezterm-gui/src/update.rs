use crate::ICON_DATA;
use anyhow::anyhow;
use config::{configuration, wezterm_version};
use http_req::request::{HttpVersion, Request};
use http_req::uri::Uri;
use mux::connui::ConnectionUI;
use serde::*;
use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use termwiz::cell::{Hyperlink, Underline};
use termwiz::color::AnsiColor;
use termwiz::escape::csi::{Cursor, Sgr};
use termwiz::escape::osc::{ITermDimension, ITermFileData, ITermProprietary};
use termwiz::escape::{OneBased, OperatingSystemCommand, CSI};
use wezterm_toast_notification::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub html_url: String,
    /// The full commit count of the published build, eg: "8718".
    pub tag_name: String,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// Where this fork publishes the newest build it has put in its APT
/// repository. The file is written by .github/workflows/publish-apt.yml.
const LATEST_RELEASE_URL: &str = "https://thracyturnix.github.io/wezterm/latest.json";
const CHANGES_URL: &str = "https://github.com/thracyturnix/wezterm/commits/main";

/// True when `latest` is a newer build of this fork than the running one.
/// Release info cached from the upstream project uses date tags, which
/// don't parse as a build count and so never count as newer.
fn is_newer(latest: &Release) -> bool {
    latest
        .tag_name
        .parse::<u64>()
        .map(|count| count > wezterm_version::wezterm_build_count())
        .unwrap_or(false)
}

fn changes_url(latest: &Release) -> String {
    if latest.html_url.starts_with("https://github.com/thracyturnix/wezterm/") {
        latest.html_url.clone()
    } else {
        CHANGES_URL.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub size: usize,
    pub url: String,
    pub browser_download_url: String,
}

fn get_github_release_info(uri: &str) -> anyhow::Result<Release> {
    let uri = Uri::try_from(uri)?;

    let mut latest = Vec::new();
    let _res = Request::new(&uri)
        .version(HttpVersion::Http10)
        .header(
            "User-Agent",
            &format!("thracyturnix-wezterm/{}", wezterm_version()),
        )
        .send(&mut latest)
        .map_err(|e| anyhow!("failed to query latest release: {}", e))?;

    /*
    println!("Status: {} {}", _res.status_code(), _res.reason());
    println!("{}", String::from_utf8_lossy(&latest));
    */

    let latest: Release = serde_json::from_slice(&latest)?;
    Ok(latest)
}

pub fn get_latest_release_info() -> anyhow::Result<Release> {
    get_github_release_info(LATEST_RELEASE_URL)
}

#[allow(unused)]
pub fn get_nightly_release_info() -> anyhow::Result<Release> {
    // This fork publishes a single rolling build.
    get_latest_release_info()
}

lazy_static::lazy_static! {
    static ref UPDATER_WINDOW: Mutex<Option<ConnectionUI>> = Mutex::new(None);
}

pub fn load_last_release_info_and_set_banner() {
    if !configuration().check_for_updates {
        return;
    }

    let update_file_name = config::DATA_DIR.join("check_update");
    if let Ok(data) = std::fs::read(update_file_name) {
        let latest: Release = match serde_json::from_slice(&data) {
            Ok(d) => d,
            Err(_) => return,
        };

        let force_ui = std::env::var_os("WEZTERM_ALWAYS_SHOW_UPDATE_UI").is_some();
        if !is_newer(&latest) && !force_ui {
            return;
        }

        set_banner_from_release_info(&latest);
    }
}

fn set_banner_from_release_info(latest: &Release) {
    let mux = crate::Mux::get();
    let url = changes_url(latest);

    let icon = ITermFileData {
        name: None,
        size: Some(ICON_DATA.len()),
        width: ITermDimension::Automatic,
        height: ITermDimension::Cells(2),
        preserve_aspect_ratio: true,
        inline: true,
        do_not_move_cursor: false,
        data: ICON_DATA.to_vec(),
    };
    let icon = OperatingSystemCommand::ITermProprietary(ITermProprietary::File(Box::new(icon)));
    let top_line_pos = CSI::Cursor(Cursor::CharacterAndLinePosition {
        line: OneBased::new(1),
        col: OneBased::new(6),
    });
    let second_line_pos = CSI::Cursor(Cursor::CharacterAndLinePosition {
        line: OneBased::new(2),
        col: OneBased::new(6),
    });
    let link_on = OperatingSystemCommand::SetHyperlink(Some(Hyperlink::new(url)));
    let underline_color = CSI::Sgr(Sgr::UnderlineColor(AnsiColor::Blue.into()));
    let underline_on = CSI::Sgr(Sgr::Underline(Underline::Single));
    let reset = CSI::Sgr(Sgr::Reset);
    let link_off = OperatingSystemCommand::SetHyperlink(None);
    mux.set_banner(Some(format!(
        "{}{}WezTerm Update Available\r\n{}{}{}{}Click to see what's new{}{}\r\n",
        icon,
        top_line_pos,
        second_line_pos,
        link_on,
        underline_color,
        underline_on,
        link_off,
        reset,
    )));
}

fn schedule_set_banner_from_release_info(latest: &Release) {
    if !is_newer(latest) {
        return;
    }
    promise::spawn::spawn_into_main_thread({
        let latest = latest.clone();
        async move {
            set_banner_from_release_info(&latest);
        }
    })
    .detach();
}

/// Returns true if the provided socket path is dead.
fn update_checker() {
    // Compute how long we should sleep for;
    // if we've never checked, give it a few seconds after the first
    // launch, otherwise compute the interval based on the time of
    // the last check.
    let update_interval = Duration::from_secs(configuration().check_for_updates_interval_seconds);
    let initial_interval = Duration::from_secs(10);

    let force_ui = std::env::var_os("WEZTERM_ALWAYS_SHOW_UPDATE_UI").is_some();

    let update_file_name = config::DATA_DIR.join("check_update");
    let delay = update_file_name
        .metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|_| ())
        .and_then(|systime| {
            let elapsed = systime.elapsed().unwrap_or(Duration::new(0, 0));
            update_interval.checked_sub(elapsed).ok_or(())
        })
        .unwrap_or(initial_interval);

    std::thread::sleep(if force_ui { initial_interval } else { delay });

    let my_sock = config::RUNTIME_DIR.join(format!("gui-sock-{}", unsafe { libc::getpid() }));

    loop {
        // Figure out which other wezterm-guis are running.
        // We have a little "consensus protocol" to decide which
        // of us will show the toast notification or show the update
        // window: the one of us that sorts first in the list will
        // own doing that, so that if there are a dozen gui processes
        // running, we don't spam the user with a lot of notifications.
        let socks = wezterm_client::discovery::discover_gui_socks();

        if configuration().check_for_updates {
            if let Ok(latest) = get_latest_release_info() {
                schedule_set_banner_from_release_info(&latest);
                if is_newer(&latest) || force_ui {
                    log::info!(
                        "latest build {} is newer than current build {}",
                        latest.tag_name,
                        wezterm_version::wezterm_build_count()
                    );

                    let url = changes_url(&latest);

                    if force_ui || socks.is_empty() || socks[0] == my_sock {
                        persistent_toast_notification_with_click_to_open_url(
                            "WezTerm Update Available",
                            "Click to see what's new",
                            &url,
                        );
                    }
                }

                config::create_user_owned_dirs(update_file_name.parent().unwrap()).ok();

                // Record the time of this check
                if let Ok(f) = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&update_file_name)
                {
                    serde_json::to_writer_pretty(f, &latest).ok();
                }
            }
        }

        std::thread::sleep(Duration::from_secs(
            configuration().check_for_updates_interval_seconds,
        ));
    }
}

pub fn start_update_checker() {
    static CHECKER_STARTED: AtomicBool = AtomicBool::new(false);
    if let Ok(false) =
        CHECKER_STARTED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
    {
        std::thread::Builder::new()
            .name("update_checker".into())
            .spawn(update_checker)
            .expect("failed to spawn update checker thread");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn release(tag: &str) -> Release {
        serde_json::from_str(&format!(r#"{{"tag_name": "{}"}}"#, tag)).unwrap()
    }

    #[test]
    fn newer_fork_build_is_detected() {
        let current = wezterm_version::wezterm_build_count();
        assert!(is_newer(&release(&(current + 1).to_string())));
        assert!(!is_newer(&release(&current.to_string())));
        assert!(!is_newer(&release(&(current - 1).to_string())));
    }

    #[test]
    fn upstream_release_tags_are_ignored() {
        assert!(!is_newer(&release("20240203-110809-5046fc22")));
        assert!(!is_newer(&release("nightly")));
        assert_eq!(changes_url(&release("1")), CHANGES_URL);
    }
}
