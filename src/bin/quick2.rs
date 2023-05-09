use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Deserialize;

use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct ObjectDigest {
    object_type: String,
    #[serde(rename = "@algorithm")]
    algorithm: String,
    #[serde(rename = "$text")]
    text: String,
}
fn save_filename(
    reader: &mut Reader<BufReader<File>>,
    element: BytesStart,
) -> Result<String, quick_xml::Error> {
    let mut element_buf = Vec::new();
    let _event = reader.read_event_into(&mut element_buf)?;
    let s = String::from_utf8(element_buf)?;
    println!("Found filename[{}]",s.trim());
    Ok(s.trim().parse().unwrap())
}

fn save_digest(
    reader: &mut Reader<BufReader<File>>,
    element: BytesStart,
    object_type: String
) -> Result<ObjectDigest, quick_xml::Error> {
    let mut algo = Cow::Borrowed("");
    for attr_result in element.attributes() {
        let a = attr_result?;
        match a.key.as_ref() {
            b"algorithm" => {
                algo = a.decode_and_unescape_value(reader)?;
                println!("Algo found {}", algo);
            },
            _ => (),
        }
    }
    let mut element_buf = Vec::new();
    let _event = reader.read_event_into(&mut element_buf)?;
    let s = String::from_utf8(element_buf)?;
    println!("Found [{}]",s.trim());
    Ok(ObjectDigest {
        object_type,
        algorithm: algo.to_string(),
        text: s.trim().parse().unwrap()
    })
}
fn main() -> Result<(), quick_xml::Error> {


    let mut reader = Reader::from_file("tests/documents/structured-metadata.xml")?;
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut count = 0;

    loop {
        let event = reader.read_event_into(&mut buf)?;

        match event {
            Event::Start(element) => match element.name().as_ref() {
                b"applicative-metadata-digest" => {
                    count += 1;
                    save_digest(&mut reader, element,"applicative_metadata".to_string())?;
                },
                b"data-object-digest" => {
                    count += 1;
                    save_digest(&mut reader, element,"data_object".to_string())?;
                },
                b"descriptive-metadata-digest" => {
                    count += 1;
                    save_digest(&mut reader, element,"descriptive_metadata".to_string())?;
                },
                b"protocol-info" => {
                    println!("protocol info found");
                    count += 1;
                },
                b"file-name" => {
                    println!("filename found");
                    count += 1;
                    save_filename(&mut reader, element)?;
                    // TODO comput and check hash of file
                },
                _ => (),
            },
            Event::Eof => break,
            _ =>(),
        }
    }
    //println!("read {} start events in total", count);
    if count != 5 {
        Err(quick_xml::Error::TextNotFound)
    }
    else {
        Ok(())
    }
}