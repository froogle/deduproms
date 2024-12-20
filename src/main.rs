use clap::{Arg, Command};
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct GameList {
    #[serde(rename = "game", default)]
    games: Vec<Game>,
}

#[derive(Debug, Deserialize)]
struct Game {
    path: String,
    name: String,
}

fn parse_gamelist(file_path: &str) -> Result<GameList, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let gamelist: GameList = from_str(&content)?;
    Ok(gamelist)
}

fn main() {
    let matches = Command::new("game_app")
        .version("1.0")
        .author("Pete Wright <peterjwright@gmail.com")
        .about("Looks in a specified directory and compares to gamelist.xml to find duplicate roms")
        .arg(
            Arg::new("gamelist")
                .short('g')
                .long("gamelist")
                .value_name("GAMELIST")
                .help("Path to the gamelist.xml file")
                .value_parser(clap::value_parser!(String))
                .default_value("gamelist.xml"),
        )
        .arg(
            Arg::new("romdir")
                .short('r')
                .long("romdir")
                .value_name("ROMDIR")
                .help("Directory where the ROM files are located")
                .value_parser(clap::value_parser!(String))
                .default_value("."),
        )
        .arg(
            Arg::new("dupdir")
                .short('d')
                .long("dupdir")
                .value_name("DUPDIR")
                .help("Directory to move duplicate files to")
                .value_parser(clap::value_parser!(String))
                .required(true),
        )
        .get_matches();

    let gamelist = matches.get_one::<String>("gamelist").unwrap();
    let romdir = matches.get_one::<String>("romdir").unwrap();
    let dupdir = matches.get_one::<String>("dupdir").unwrap();

    println!("Gamelist path: {}", gamelist);
    println!("ROM directory: {}", romdir);
    println!("Duplicate directory: {}", dupdir);

    match parse_gamelist(gamelist) {
        Ok(gamelist) => {
            let mut game_map: HashMap<String, Vec<String>> = HashMap::new();
            for game in gamelist.games {
                let filename = Path::new(&game.path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                game_map
                    .entry(game.name.clone())
                    .or_default()
                    .push(filename);
            }

            for (name, paths) in game_map {
                if paths.len() > 1 {
                    let mut duplicates = Vec::new();
                    for path in &paths {
                        let rom_path = PathBuf::from(romdir).join(path);
                        if rom_path.exists() {
                            duplicates.push(rom_path);
                        }
                    }
                    if duplicates.len() > 1 {
                        println!("Duplicate game: {}", name);
                        for (i, path) in duplicates.iter().enumerate() {
                            println!("  {}: {}", i + 1, path.display());
                        }

                        print!("Enter the number of the file to keep (default is 1): ");
                        io::stdout().flush().unwrap();

                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let choice: usize = input.trim().parse().unwrap_or(1);

                        for (i, path) in duplicates.iter().enumerate() {
                            if i + 1 != choice {
                                let dest_path =
                                    PathBuf::from(dupdir).join(path.file_name().unwrap());
                                fs::rename(path, dest_path).unwrap();
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Error parsing gamelist: {}", e),
    }
}
