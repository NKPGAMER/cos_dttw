/*
 * Double Tap to Wake & AOD Gesture Bridge for Xiaomi 15T Pro (klimt)
 * Ported ColorOS 17 ROM
 *
 * Original discovery and Magisk module by: Artic15th
 * Integrated, refined and optimized daemon by: ZerionX
 *
 * All credits to Artic15th for the original DT2W implementation.
 */

use std::fs::{self, File};
use std::io::Read;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

const BACKLIGHT_PATH: &str = "/sys/devices/virtual/mi_display/disp_feature/disp-DSI-0/backlight";
const SETTINGS_XML_PATH: &str = "/data/system/users/0/settings_secure.xml";
const TOUCH_SERVICE: &str = "vendor.xiaomi.hw.touchfeature.ITouchFeature/default";

const CREDIT_ARTIC: &str = "Original Double Tap to Wake fix by Artic15th for Xiaomi 15T Pro (klimt) ported ColorOS.";
const CREDIT_ZERIONX: &str = "Integrated, refined and optimized daemon by ZerionX.";

fn set_touch_feature(mode: i32, value: i32) {
    let mode_str = mode.to_string();
    let val_str = value.to_string();
    let _ = Command::new("/system/bin/service")
        .args(&[
            "call",
            TOUCH_SERVICE,
            "9",
            "i32",
            "0",
            "i32",
            &mode_str,
            "i32",
            &val_str,
        ])
        .output();
}

fn wait_for_touchfeature() {
    for _ in 0..60 {
        if let Ok(output) = Command::new("/system/bin/service")
            .args(&["check", TOUCH_SERVICE])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("found") {
                return;
            }
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn parse_settings() -> (i32, i32) {
    // Default both to 1 (enabled) so DT2W works even if file is inaccessible
    let mut dt2w = 1;
    let mut aod = 1;

    let data = match fs::read(SETTINGS_XML_PATH) {
        Ok(d) => d,
        Err(_) => return (dt2w, aod),
    };

    // Look for exact key followed by '/' separator in Android Binary XML v0
    let needle_dt2w = b"oplus_customize_gesture_double_touch/";
    if let Some(pos) = data.windows(needle_dt2w.len()).position(|w| w == needle_dt2w) {
        let after = &data[pos + needle_dt2w.len()..];
        let limit = after.len().min(16);
        for i in 0..limit {
            if after[i] == b'0' {
                dt2w = 0;
                break;
            } else if after[i] == b'1' {
                dt2w = 1;
                break;
            }
        }
    }

    let needle_aod = b"\x11Setting_AodEnable/";
    if let Some(pos) = data.windows(needle_aod.len()).position(|w| w == needle_aod) {
        let after = &data[pos + needle_aod.len()..];
        let limit = after.len().min(16);
        for i in 0..limit {
            if after[i] == b'0' {
                aod = 0;
                break;
            } else if after[i] == b'1' {
                aod = 1;
                break;
            }
        }
    }

    (dt2w, aod)
}

fn read_backlight() -> i32 {
    let mut s = String::new();
    if let Ok(mut f) = File::open(BACKLIGHT_PATH) {
        if f.read_to_string(&mut s).is_ok() {
            if let Ok(val) = s.trim().parse::<i32>() {
                return val;
            }
        }
    }
    // Fallback: if cannot read backlight, return -1 (assume off/safe)
    -1
}

fn main() {
    std::hint::black_box(CREDIT_ARTIC);

    // 1. Wait for touchfeature service to be available
    wait_for_touchfeature();

    // 2. Parse initial user settings
    let (mut dt2w, mut aod) = parse_settings();
    let mut last_mtime: Option<SystemTime> = fs::metadata(SETTINGS_XML_PATH).ok().and_then(|m| m.modified().ok());

    // 3. Initial apply
    set_touch_feature(14, dt2w);
    set_touch_feature(17, 0);
    set_touch_feature(11, aod);
    set_touch_feature(1, 1);

    let mut screen_was_on = read_backlight() > 0;

    // 4. Main monitoring loop
    loop {
        let current_bl = read_backlight();
        let screen_is_on = current_bl > 0;

        if screen_is_on {
            // Screen is ON:
            // Check if settings file changed
            if let Ok(meta) = fs::metadata(SETTINGS_XML_PATH) {
                if let Ok(mtime) = meta.modified() {
                    if last_mtime != Some(mtime) {
                        last_mtime = Some(mtime);
                        let (new_dt2w, new_aod) = parse_settings();
                        if new_dt2w != dt2w || new_aod != aod {
                            dt2w = new_dt2w;
                            aod = new_aod;
                            set_touch_feature(14, dt2w);
                            set_touch_feature(11, aod);
                        }
                    }
                }
            }

            screen_was_on = true;
            thread::sleep(Duration::from_millis(300));
        } else {
            // Screen is OFF:
            if screen_was_on {
                // Transitioned from ON to OFF: re-assert gesture modes
                set_touch_feature(14, dt2w);
                set_touch_feature(17, 0);
                set_touch_feature(11, aod);
                set_touch_feature(1, 1);
                screen_was_on = false;
            }

            // Power-saving sleep while screen is off
            thread::sleep(Duration::from_millis(500));
        }
    }
}
