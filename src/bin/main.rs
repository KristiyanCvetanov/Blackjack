use ggez::{
    ContextBuilder,
    conf::{Conf, WindowMode},
    event,
    filesystem,
};

use std::io::{BufReader, Write, BufWriter};
use std::fs::File;
use std::fs::OpenOptions;

use std::env;
use std::path;

use blackjack::main_state;

const FILE_NAME: &str = "stats.txt";
const ICON_PATH: &str = "\\icons\\black-jack.png";

fn create_file(file_name: &str) -> File {
    {
        let _ = File::create(file_name).unwrap();
    }
    {
        let f = OpenOptions::new().write(true).open(file_name).unwrap();
        let mut writer = BufWriter::new(f);

        writer.write(b"0 0 0").unwrap();
        writer.flush().unwrap();
    }

    File::open(file_name).unwrap()
}

fn main() {
    let mut conf = Conf::new().
        window_mode(WindowMode {
            width: 1900.0,
            height: 900.0,
            ..Default::default()
        });
      
    conf.window_setup = conf.window_setup.title("Blackjack").icon(ICON_PATH);    

    let (mut ctx, event_loop) = ContextBuilder::new("BlackJack", "Kris").
        default_conf(conf.clone()).
        build().
        unwrap();

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        filesystem::mount(&mut ctx, &path, true);
    }

    let f;
    let f_unwrapped = File::open(FILE_NAME);

    match f_unwrapped {
        Ok(file) => f = file,
        Err(_) => f = create_file(FILE_NAME),
    }
    let reader = BufReader::new(f);

    let state = main_state::MainState::new(&mut ctx, reader, FILE_NAME).unwrap();

    event::run(ctx, event_loop, state);
}