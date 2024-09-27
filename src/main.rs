use std::{
    env,
    error::Error,
    fs, path::PathBuf,
    process::{exit, Command, Stdio},
};
use structs::{Message, Playlist};
use tokio::sync::mpsc;
use utils::{download, get_ffmpeg_txt};
use ytd_rs::Arg;

mod structs;
mod utils;

static LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel::<Message>(32);
    tokio::spawn(async move {
        loop {
            while let Some(message) = rx.recv().await {
                println!("[LOG] {}", message.get_content())
            }
        }
    });

    let current_dir = env::current_dir().unwrap();
    let config_path: PathBuf = [current_dir.clone(), ".playlist.json".into()]
        .iter()
        .collect();

    if let Ok(config) = fs::read_to_string(&config_path) {
        let playlist = match serde_json::from_str::<Playlist>(config.as_str()) {
            Ok(playlist) => playlist,
            Err(e) => {
                eprintln!("Error while parsing config: {}", e);
                exit(-1);
            }
        };

        match fs::create_dir(&current_dir) {
            Ok(()) => println!("Created playlist dir"),
            Err(e) => eprintln!("{e}"),
        }

        let mut thread_handles = vec![];

        for person in playlist.people {
            let letter = LETTERS[person.index];
            println!("{letter}");
            let format = format!("%(playlist_index)02d{}.%(ext)s", letter);
            let current_dir = current_dir.clone();

            let args = vec![
                Arg::new_with_arg("-o", &format),
            ];

            let handle = tokio::spawn(download(tx.clone(), current_dir, args, person));
            thread_handles.push(handle);
        }

        for handle in thread_handles.into_iter() {
            let join_result = handle.await;

            if let Err(e) = join_result {
                eprintln!("{e}");
            }
        }

        let txt_location: PathBuf = [current_dir, "videos.txt".into()].iter().collect();
        let final_string = get_ffmpeg_txt()?;
        fs::write(&txt_location, final_string)?;

        let ffmpeg_command = Command::new("ffmpeg")
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&txt_location)
            .arg(format!("Playlist {}.{}", playlist.number, "mp4"))
            .stdout(Stdio::piped())
            .output();

        if let Ok(output) = ffmpeg_command {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        eprintln!("No playlist config found.. writing one");
        let content = serde_json::to_string_pretty(&Playlist::default())?;
        match fs::write(&config_path, content) {
            Ok(()) => println!(
                "Wrote playlist config at {} successfully",
                config_path.to_string_lossy()
            ),
            Err(e) => println!("{e}"),
        }
    }

    Ok(())
}
