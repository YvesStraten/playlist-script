use std::{env, fs, io, path::PathBuf};

use tokio::sync::mpsc::Sender;
use ytd_rs::{error::YoutubeDLError, Arg, YoutubeDL};

use crate::structs::{Message, Person};

pub fn get_ffmpeg_txt() -> Result<String, io::Error> {
    let current_dir = env::current_dir()?;
    let dir = fs::read_dir(&current_dir)?;

    let mut file_names = vec![];
    for file in dir {
        let file = file?;
        let file_name = &file.file_name().into_string();

        if let Ok(name) = file_name {
            let formatted_filename = format!("file {}'", name);
            file_names.push(formatted_filename);
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
    args: Vec<Arg>,
    person: Person,
) -> Result<(), YoutubeDLError> {
    match YoutubeDL::new(&current_dir, args, &person.link) {
        Ok(instance) => match instance.download() {
            Ok(_) => {
                let _ = tx.send(Message::Progress(format!("Downloaded playlist {}", person.link).to_string())).await;
                Ok(())
            }

            Err(e) => Err(e),
        },

        Err(e) => Err(e),
    }
}

