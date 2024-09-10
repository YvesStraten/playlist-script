use std::{env, error::Error, fs::{self}, path::PathBuf, process::{Command, Stdio}, sync::mpsc, thread};
use serde::{Deserialize, Serialize};
use ytd_rs::{Arg, YoutubeDL};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Playlist {
    number: u8, 
    people: Vec<Person>
}

impl Default for Playlist {
    fn default() -> Self {
        Self { number: 0, people: vec![
            Person {
                index: 0,
                link: String::new()
            }, 
            Person {
                index: 1, link: String::new()
            },

            Person {
                index: 2, link: String::new()
            }
        ] }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Person {
    index: usize, 
    link: String
}

static LETTERS: [&str; 26] = [ "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", 
    "L", "M", "N", "O", "P", 
    "Q", "R", "S", "T", "U", 
    "V", "W", "X", "Y", "Z"];

fn main() -> Result<(), Box<dyn Error>>{
    let (tx, rx) = mpsc::channel::<String>();

    let listener_thread = thread::spawn(move ||{
        loop {
        if let Ok(message) = rx.try_recv() {
            println!("{message}")
        }

        }
    });

    let current_dir = env::current_dir().unwrap();
    let config_path: PathBuf = [current_dir.clone(), ".playlist.json".into()].iter().collect();
    
    if let Ok(config) = fs::read_to_string(&config_path) {
        let playlist: Playlist = serde_json::from_str(config.as_str())?;

        match fs::create_dir(&current_dir) {
            Ok(()) => println!("Created playlist dir"), 
            Err(e) => eprintln!("{e}")
        }

        let mut thread_handles = vec![];

        for person in playlist.people {
            let tx = tx.clone();
            let letter = LETTERS[person.index];
            println!("{letter}");
            let format = format!("%(playlist_index)02d{}.%(ext)s", letter);
            let current_dir = current_dir.clone();

            let args = vec![
                Arg::new_with_arg("-S", "res,ext:mp4,m4a"),
                Arg::new_with_arg("-o", &format)
            ];

            let new_thread = thread::spawn(move ||{
                let ytd = YoutubeDL::new(&current_dir, args, &person.link);
                if let Ok(ytd) = ytd {
                    match ytd.download() {
                        Ok(_) => {
                            let _ = tx.send(format!("Done with playlist letter {}", letter));
                        }, 

                        Err(e) => eprintln!("{e}")
                    }

                }

            });

            thread_handles.push(new_thread);
        }

        for handle in thread_handles.into_iter() {
            let _ = handle.join();
        }

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

        dbg!(&file_names);

        let mut txt_location = current_dir.clone(); 
        txt_location.push("videos.txt");

        let final_string: String = file_names.into_iter().map(|name| {
            let string = format!("{} \n", name);
            string
        }).collect::<String>();

        println!("{final_string}");

        fs::write(&txt_location, final_string)?;

        let ffmpeg_command = Command::new("ffmpeg") 
            .arg("-f").arg("concat").arg("-safe").arg("0")
        .arg("-i").arg(&txt_location).arg(format!("Playlist {}.{}", playlist.number, "mp4")).stdout(Stdio::piped())
        .spawn().expect("Failed to run ffmpeg");

    } else {
        let content = serde_json::to_string(&Playlist::default())?;
        fs::write(&config_path, content)?;
    }

    Ok(())
} 
