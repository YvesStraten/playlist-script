use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    process::{exit, Stdio},
};
use structs::{Message, Playlist};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc,
};
use utils::{download, get_ffmpeg_txt};

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
            let current_dir = current_dir.clone();

            let handle = tokio::spawn(download(tx.clone(), current_dir, person, letter));
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

        let mut ffmpeg_command = Command::new("ffmpeg");

        ffmpeg_command.arg("-f");
        ffmpeg_command.arg("concat");
        ffmpeg_command.arg("-i");
        ffmpeg_command.arg(&txt_location);
        ffmpeg_command.arg("-filter:a");
        ffmpeg_command.arg("loudnorm");
        ffmpeg_command.arg(format!("Playlist {}.{}", playlist.number, playlist.format));
        ffmpeg_command.stdout(Stdio::piped());

        let mut child = ffmpeg_command.spawn().expect("Did not find ffmpeg!");
        let stdout = child.stdout.take().expect("Not stdout to take!");

        let mut reader = BufReader::new(stdout).lines();

        tokio::spawn(async move {
            let _ = child.wait().await;
        });

        while let Some(outline) = reader.next_line().await? {
            let _ = tx.send(Message::Progress(outline)).await;
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
