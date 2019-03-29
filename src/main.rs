extern crate cursive;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serializable_enum;
extern crate clap;
extern crate crossbeam_channel;

mod ui;
mod paperspace;

use std::thread;

use clap::{Arg, App};

use paperspace::Paperspace;
use ui::Ui;

fn main() {
    let options = App::new("paperpi")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Miroslav Prasil")
        .about("Tool that can be used to start and stop your Paperspace machine")
        .arg(Arg::with_name("token")
            .help("Paperspace API token")
            .required(true)
            .index(1))
        .get_matches();


    let (ps_tx, ps_rx) = crossbeam_channel::unbounded();
    let (ui_tx, ui_rx) = crossbeam_channel::unbounded();

    let paperspace = Paperspace::new(
        &String::from(options.value_of("token").unwrap())
    );
    thread::spawn(move || {
        paperspace.run(ps_rx, ui_tx);
    });

    let mut ui = Ui::new(ui_rx, ps_tx);
    ui.run();

}