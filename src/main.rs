extern crate flvparser;
extern crate clap;
#[macro_use]
extern crate prettytable;

use std::fs::File;
use std::io::{BufReader, Read};

use flvparser::{FLVFile, parse_flv_file, FLVTagType};
use clap::{App, Arg};
use prettytable::{Table, Row, Cell, Attr, format};

pub fn print_table(flv_file: &FLVFile, print_body: bool) {
    let mut header = Table::new();
    header.set_titles(Row::new(vec![
        Cell::new("FLV File Header").with_style(Attr::Bold),
    ]));
    header.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    header.add_row(row!(
        "Signature (3B)",
        &format!(
            "{:x} {:x} {:x}",
            flv_file.header.signature[0],
            flv_file.header.signature[1],
            flv_file.header.signature[2]
        )
    ));
    header.add_row(row!(
        "Version (1B)",
        &format!("{}", flv_file.header.version)
    ));
    header.add_row(row!(
        "Flags (1B)",
        &format!(
            "{:04b} {:04b}",
            flv_file.header.flags & 0xf0,
            flv_file.header.flags & 0x0f
        )
    ));
    header.add_row(row!(
        "DataOffset (4B)",
        &format!("{}", flv_file.header.data_offset)
    ));
    header.printstd();

    let mut body = Table::new();
    body.set_titles(Row::new(vec![
        Cell::new("FLV File Body").with_style(Attr::Bold),
    ]));
    body.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    body.add_row(row!(
        "Index",
        "TagType (1B)",
        "DataSize (3B)",
        "Timestamp (4B)",
        "StreamID (3B)"
    ));
    let mut index = 0usize;
    let mut script_tag_num = 0usize;
    let mut video_tag_num = 0usize;
    let mut audio_tag_num = 0usize;
    for (tag, _) in &flv_file.body.tags {
        index += 1;
        match tag.header.tag_type {
            FLVTagType::Script => script_tag_num += 1,
            FLVTagType::Video => video_tag_num += 1,
            FLVTagType::Audio => audio_tag_num += 1,
        }
        body.add_row(Row::new(vec![
            Cell::new(&format!("{}", index)),
            Cell::new(&format!("{:?}", tag.header.tag_type)),
            Cell::new(&format!("{}", tag.header.data_size)),
            Cell::new(&format!("{}", tag.header.timestamp)),
            Cell::new(&format!("{}", tag.header.stream_id)),
        ]));
    }
    if print_body {
        body.printstd();
    }

    let mut result = Table::new();
    result.set_titles(Row::new(vec![
        Cell::new("Total tag number").with_style(Attr::Bold),
        Cell::new("Script tag number").with_style(Attr::Bold),
        Cell::new("Video tag number").with_style(Attr::Bold),
        Cell::new("Audio tag number").with_style(Attr::Bold),
    ]));
    result.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    result.add_row(row!(
        &format!("{}", index),
        &format!("{}", script_tag_num),
        &format!("{}", video_tag_num),
        &format!("{}", audio_tag_num),
    ));
    result.printstd();
}

fn main() {
    let matches = App::new("FLV File Parser")
        .version("0.1.0")
        .author("Qinxuan Chen <koushiro.cqx@gmail.com>")
        .about("A FLV File parser written in Rust with the nom framework")
        .arg(
            Arg::with_name("INPUT")
                .help("The input FLV file to parse")
                .required(true),
        ).arg(
            Arg::with_name("print")
                .short("p")
                .long("print")
                .help("Prints all tables about FLV File info"),
        ).get_matches();

    let file_path = matches.value_of("INPUT").unwrap();
    let file = File::open(file_path).expect("Unable to open the file");
    let mut reader = BufReader::new(file);
    let mut contents = vec![];
    reader.read_to_end(&mut contents).unwrap();

    let flv_file: FLVFile = parse_flv_file(&contents).unwrap().1;

    if matches.is_present("print") {
        print_table(&flv_file, true);
    } else {
        print_table(&flv_file, false);
    }
}
