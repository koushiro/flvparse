use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use clap::Parser;
use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets};
use flvparse::{FlvFile, FlvTagType};

#[derive(Debug, Parser)]
#[command(author, about)]
struct Cli {
    /// The input FLV file to parse.
    #[arg(short, long)]
    input: PathBuf,
    /// Prints all tables about FLV File info.
    #[arg(short, long)]
    print: bool,
}

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let file = File::open(cli.input)?;
    let mut reader = BufReader::new(file);
    let mut input = vec![];
    reader.read_to_end(&mut input)?;

    let (_, flv) = FlvFile::parse(&input).map_err(|e| anyhow!("failed to parse: {e:?}"))?;
    if cli.print {
        print_table(&flv, true);
    } else {
        print_table(&flv, false);
    }
    Ok(())
}

fn print_table(flv_file: &FlvFile, print_body: bool) {
    println!("FLV File Header");
    let mut header = Table::new();
    header.load_style(presets::UTF8_BORDERS_ONLY);
    header.set_content_arrangement(ContentArrangement::Dynamic);
    header.set_header(vec![
        Cell::new("Field").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);
    header.add_row(vec![
        Cell::new("Signature (3B)"),
        Cell::new(format!(
            "{:x} {:x} {:x}",
            flv_file.header.signature[0],
            flv_file.header.signature[1],
            flv_file.header.signature[2]
        )),
    ]);
    header.add_row(vec![
        Cell::new("Version (1B)"),
        Cell::new(format!("{}", flv_file.header.version)),
    ]);
    header.add_row(vec![
        Cell::new("Flags (1B)"),
        Cell::new(format!(
            "{:04b} {:04b}",
            flv_file.header.flags & 0xf0,
            flv_file.header.flags & 0x0f
        )),
    ]);
    header.add_row(vec![
        Cell::new("DataOffset (4B)"),
        Cell::new(format!("{}", flv_file.header.data_offset)),
    ]);
    println!("{header}");

    let mut body = Table::new();
    body.load_style(presets::UTF8_BORDERS_ONLY);
    body.set_content_arrangement(ContentArrangement::Dynamic);
    body.set_header(vec![
        Cell::new("Index").add_attribute(Attribute::Bold),
        Cell::new("TagType (1B)").add_attribute(Attribute::Bold),
        Cell::new("DataSize (3B)").add_attribute(Attribute::Bold),
        Cell::new("Timestamp (4B)").add_attribute(Attribute::Bold),
        Cell::new("StreamID (3B)").add_attribute(Attribute::Bold),
    ]);
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
        body.add_row(vec![
            Cell::new(format!("{}", index)),
            Cell::new(format!("{:?}", tag.header.tag_type)),
            Cell::new(format!("{}", tag.header.data_size)),
            Cell::new(format!("{}", tag.header.timestamp)),
            Cell::new(format!("{}", tag.header.stream_id)),
        ]);
    }
    if print_body {
        println!("FLV File Body");
        println!("{body}");
    }

    println!("Tag Summary");
    let mut result = Table::new();
    result.load_style(presets::UTF8_BORDERS_ONLY);
    result.set_content_arrangement(ContentArrangement::Dynamic);
    result.set_header(vec![
        Cell::new("Total tag number").add_attribute(Attribute::Bold),
        Cell::new("Script tag number").add_attribute(Attribute::Bold),
        Cell::new("Video tag number").add_attribute(Attribute::Bold),
        Cell::new("Audio tag number").add_attribute(Attribute::Bold),
    ]);
    result.add_row(vec![
        Cell::new(format!("{}", index)),
        Cell::new(format!("{}", script_tag_num)),
        Cell::new(format!("{}", video_tag_num)),
        Cell::new(format!("{}", audio_tag_num)),
    ]);
    println!("{result}");
}
