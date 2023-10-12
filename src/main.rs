// this COULD run flake update and compare dates, BUT I don't want to
// because then I would have to figure out how to check less often and consume less compute

#[derive(serde::Deserialize, Clone, Copy)]
struct Configuration {
    lock_path: String,
    inputs: Vec<String>,
    threshold: i32,
    // you COULD run a command every time you update. however, this would be somewhat wasteful because this thing is going to be running constantly
    // command: Option<String>,
}

fn main() {
    // wtf does tauri do here

    // be constantly checking if you're up to date by deserializing the flake lockfile and checking if you're many whole days away from the last update
    // update the taskbar icon

    println!("Hello, world!");
}

// notifcation taskbar icon stuff, check every day at a certain time?

// get config information, which inputs you care about, how many days to warn the user, update command
fn get_config() -> anyhow::Result<Configuration> {
    // get XDG config path
    // get file
    // toml deserialize file into Configuration

    Ok(config)
}

// if older than this, be alarmed
const DAY_THRESHOLD: i32 = 3;
const HIGHLIGHTED_INPUT: &str = "nixpkgs";

// ❯ date -d @(bat testlock.lock | jq '.nodes[.nodes.root.inputs.master].locked.lastModified')
fn flake_last_updated() -> anyhow::Result<()> {
    // read in flake.lock from location
    // deserialize into a struct (what is the format?)

    // iterate through and find the most recent modified date in all inputs
    // the user is expected to not update only specific entries in the flake so
    // we can just take the most recent thing as an indication of when the flake was last updated

    Ok(())
}
