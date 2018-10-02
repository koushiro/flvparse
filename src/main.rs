extern crate flvparser;

#[macro_use]
extern crate prettytable;

use flvparser::{FLVFile, parse_flv_file};

use prettytable::{Table, Row, Cell, Attr, format};

fn main() {
    let flv_file: FLVFile = parse_flv_file(include_bytes!("../assets/test.flv"))
        .unwrap()
        .1;

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
        "TagType (1B)",
        "DataSize (3B)",
        "Timestamp (4B)",
        "StreamID (3B)"
    ));
    for (tag, _) in flv_file.body.tags {
        body.add_row(Row::new(vec![
            Cell::new(&format!("{:?}", tag.header.tag_type)),
            Cell::new(&format!("{}", tag.header.data_size)),
            Cell::new(&format!("{}", tag.header.timestamp)),
            Cell::new(&format!("{}", tag.header.stream_id)),
        ]));
    }
    body.printstd();
}
