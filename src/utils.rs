use std::{
    env, fs,
    path::{self, PathBuf},
    process::Stdio,
};

use ez_ffmpeg::Input;
use tokio::{io::{self, AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::Sender};

use crate::structs::{Message, Person};
use letter_gen::LetterSequence;

pub fn get_ffmpeg_files() -> Result<Vec<Input>, io::Error> {
    let current_dir = env::current_dir()?;
    let dir = fs::read_dir(&current_dir)?;

    let mut file_names = vec![];
    for file in dir {
        let file = file?;
        let file_name = file.file_name().into_string();

        if let Ok(name) = file_name {
            let ext = path::Path::new(&name).extension();
            if let Some(ext) = ext {
                if ext != "json" {
                    file_names.push(name);
                }
            }
        }
    }

    file_names.sort();
    let final_string: Vec<Input> = file_names
        .into_iter()
        .map(|name| { println!("{name}"); let input: Input = name.into();
            input.set_hwaccel("cuda").set_video_codec("h264_cuvid")
        })
        .collect();

    Ok(final_string)
}

pub async fn download(
    tx: Sender<Message>,
    current_dir: PathBuf,
    person: Person,
    letter: LetterSequence,
) -> io::Result<()> {
    let mut command = Command::new("yt-dlp");
    command.arg(&person.link);
    let format = format!("%(playlist_index)02d{}.%(ext)s", letter);
    command.arg("-o");
    command.arg(format);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());

    let mut child = command.spawn().expect("Failed to spawn yt-dlp!");
    let stdout = child.stdout.take().expect("No handle to stdout!");

    let mut reader = BufReader::new(stdout).lines();

    let tx2 = tx.clone();
    let _ = tx2.send(Message::Progress(format!("Downloading {}", &person.link))).await;
    tokio::spawn(async move {
        let status = child.wait().await;

        if let Ok(code) = status {
            if code.success() {
                let _ = tx2.send(Message::Progress("DONE!".to_string())).await;
            }
        }
    });

    while let Some(outline) = reader.next_line().await? {
        let _ = tx.send(Message::Progress(outline)).await;
    }

    Ok(())
}
