use std::{
    env, fs,
    path::{self, PathBuf},
    process::Stdio,
};

use tokio::{io::{self, AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::Sender};

use crate::structs::{Message, Person};

pub fn get_ffmpeg_txt() -> Result<String, io::Error> {
    let current_dir = env::current_dir()?;
    let dir = fs::read_dir(&current_dir)?;

    let mut file_names = vec![];
    for file in dir {
        let file = file?;
        let file_name = &file.file_name().into_string();

        if let Ok(name) = file_name {
            let ext = path::Path::new(name).extension();
            if let Some(ext) = ext {
                if ext != "json" {
                    let formatted_filename = format!("file {}'", name);
                    file_names.push(formatted_filename);
                }
            }
        }
    }

    let final_string: String = file_names
        .into_iter()
        .map(|name| {
            let string = format!("{} \n", name);
            string
        })
        .collect::<String>();

    println!("{final_string}");

    Ok(final_string)
}

pub async fn download(
    tx: Sender<Message>,
    current_dir: PathBuf,
    person: Person,
    letter: &str,
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
                let _ = tx2.send(Message::Progress("DONE!".to_string()));
                
            }
            
        }
    });

    while let Some(outline) = reader.next_line().await? {
        let _ = tx.send(Message::Progress(outline)).await;
    }

    Ok(())
}
