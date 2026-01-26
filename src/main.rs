// this COULD run flake update and compare dates, BUT I don't want to
// because then I would have to figure out how to check less often and consume less compute

// simplest solution: when the system gets rebuilt it takes information from the flake.lock and
// commits it. the module for this will take that info and put it in here to include it as a constant.

include!("modified_data.rs");

mod reboot_check;

#[derive(serde::Serialize)]
pub enum State {
    Info,
    Good,
    Warning,
    Critical,
}

#[derive(serde::Serialize)]
pub struct BarCommand {
    icon: String,
    state: State,
    text: String,
}

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let now = chrono::Utc::now();

    let time = chrono::DateTime::from_timestamp(MODIFIED_DATE, 0)
        .context("Could not deserialize timestamp. Corrupted flake?")?;

    let duration_days = now.signed_duration_since(time).num_days();

    let mut status: State;

    if duration_days >= OUT_OF_DATE_THRESHOLD {
        // it is critical that you update
        status = State::Critical;
    } else if duration_days >= UPDATE_THRESHOLD {
        // warn to update
        status = State::Warning;
    } else if duration_days <= GOOD_THRESHOLD {
        // you don't need to update yet
        status = State::Good;
    } else {
        unreachable!("all possible values of duration_days are handled");
    }

    let mut text = format!("Age: {}", duration_days);

    // Check if reboot is needed due to kernel/module version changes
    if let Ok(mismatches) = reboot_check::check_reboot_needed() {
        if !mismatches.is_empty() {
            status = State::Critical;
            let reboot_info: Vec<String> = mismatches
                .iter()
                .map(|m| format!("{} {}→{}", m.name, m.booted, m.current))
                .collect();
            text = format!("{} | Reboot: {}", text, reboot_info.join(", "));
        }
    }

    let code = BarCommand {
        icon: STATUS_ICON.to_string(),
        state: status,
        text,
    };

    println!(
        "{}",
        serde_json::to_string(&code).context("Could not serialize status")?
    );

    Ok(())
}
