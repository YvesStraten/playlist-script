use std::{
    env, error::Error, fs, path::PathBuf, process::{exit, Stdio}, sync::{Arc, Mutex}, time::Duration
};
use ez_ffmpeg::{FfmpegContext, Output};
use letter_gen::LetterGenerator;
use structs::{Message, Playlist};
use tokio::{
    sync::mpsc, task::JoinSet,
};
use tokio_stream::StreamExt;
use utils::{download, get_ffmpeg_files};

mod structs;
mod utils;

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

        let mut join_set = JoinSet::new();
        let mut generator = LetterGenerator::new();

        for person in playlist.people {
            let current_dir = current_dir.clone();
            let letter = generator.next().await.unwrap();


            let _ = join_set.spawn(download(tx.clone(), current_dir, person, letter));
        }

        for join_result in join_set.join_all().await {
            if let Err(e) = join_result {
                eprintln!("{e}");
            }
        }

        let ffmpeg_files = get_ffmpeg_files()?;
        let output: Output = format!("Playlist_{}.{}", playlist.number, playlist.format).into();
        let output = output.set_video_codec("h264_nvenc");

        FfmpegContext::builder()
            .inputs(ffmpeg_files)
            .filter_desc("concat=n=3:v=1:a=1")
            // .filter_desc("loudnorm")
            .output(output)
            .build().unwrap()
            .start().unwrap()
            .await?;
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
