// Copyright 2019-2020 koushiro. Licensed under MIT.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use flvparser::{parse, FlvFile, FlvTagType};
use prettytable::{cell, format, row, Attr, Cell, Row, Table};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(author, about)]
struct Opt {
    /// The input FLV file to parse.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,
    /// Prints all tables about FLV File info.
    #[structopt(short = "p", long)]
    print: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Opt = Opt::from_args();

    let file = File::open(opt.input)?;
    let mut reader = BufReader::new(file);
    let mut contents = vec![];
    reader.read_to_end(&mut contents)?;

    let flv = parse(&contents)?;
    if opt.print {
        print_table(&flv, true);
    } else {
        print_table(&flv, false);
    }
    Ok(())
}

fn print_table(flv_file: &FlvFile, print_body: bool) {
    let mut header = Table::new();
    header.set_titles(Row::new(vec![
        Cell::new("FLV File Header").with_style(Attr::Bold)
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
        Cell::new("FLV File Body").with_style(Attr::Bold)
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
            FlvTagType::Script => script_tag_num += 1,
            FlvTagType::Video => video_tag_num += 1,
            FlvTagType::Audio => audio_tag_num += 1,
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
