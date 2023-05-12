use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use serde::Deserialize;


//use ring::digest::{Context, Digest, SHA256};

use chrono::{DateTime, Datelike};
use chrono::{Duration, Utc};

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct ObjectDigest {
    object_type: String,
    #[serde(rename = "@algorithm")]
    algorithm: String,
    #[serde(rename = "$text")]
    text: String,
}
fn ymd(s: &str) -> (i32,u32,u32) {
    let rfc = DateTime::parse_from_rfc3339(s).unwrap();
    let year = rfc.year();
    let month = rfc.month();
    let day = rfc.day();
    (year, month, day)
}
fn save_sum_up(
    reader: &mut Reader<BufReader<File>>,
    element: BytesStart,
) -> Result<String, quick_xml::Error> {
    let mut d_year: i32 = 0;
    let mut d_month: u32 = 0;
    let mut d_day: u32 = 0;
    let mut e_year: i32 = 0;
    let mut e_month: u32 = 0;
    let mut e_day: u32 = 0;
    let mut attrs = HashMap::new();
    let mut total = 0;
    let sum_up: Vec<(String,String)> = element
        .attributes()
        .map(|att_result| {
            match att_result {
                Ok(a) => {
                    let key = reader.decoder().decode(a.key.local_name().as_ref())
                        .unwrap().to_string();
                    let value = a.decode_and_unescape_value(&reader)
                        .unwrap().to_string();
                    match key.as_str() {
                        "deposit-date" => {
                            let (d_year, d_month, d_day) = ymd(value.as_str());
                            let fmt = format!("{}/{}/{}",d_day,d_month,d_year);
                            println!("Deposit date: {}",fmt);
                            total += 1;
                        },
                        "end-of-life-date" => {
                            (e_year, e_month, e_day) = ymd(value.as_str());
                            let fmt = format!("{}/{}/{}",e_day,e_month,e_year);
                            println!("End of life date: {}",fmt);
                            total += 2;
                        },
                        "uri" => {
                            println!("{}: {}",key,value);
                            total += 4;
                        }
                        _ => ()
                    }
                    attrs.insert(key.clone(), value.clone());
                    (key, value)
                },
                Err(_e) => {
                    (String::new(), String::new())
                }
            }
    })
        .collect();
    //println!("Sum-up : {:#?}",sum_up);
    // TODO check deposit-date and end-of-life-date
    if total == 7 {
        let d_date = attrs.get("deposit-date").unwrap();
        let e_date = attrs.get("end-of-life-date").unwrap();
        let uri = attrs.get("uri").unwrap();
        println!("Two dates and one uri found");
        let actual = Utc::now().timestamp();
        let end = DateTime::parse_from_rfc3339(e_date.as_str()).unwrap().timestamp();
        let end2 = DateTime::parse_from_rfc3339("2023-01-01T12:00:00.0Z").unwrap().timestamp();
        if end2 < actual {
            println!("end date reached");
        }
        else {
            println!("end date not reached");
        }
    }

    Ok("OK".to_string())
}
fn add_dc(reader: &mut Reader<BufReader<File>>, hm: &mut HashMap<String, String>, what: String) {
    let mut element_buf = Vec::new();
    let _event = reader.read_event_into(&mut element_buf).unwrap();
    let s = String::from_utf8(element_buf).unwrap();
    //println!("Need to add value {} to key {}",s.trim(), what);
    if hm.get(what.as_str()).is_some() {
        println!("--- Key {} already exists", what);
    }
    else {
        hm.insert(what, s);
    }
}
fn save_filename(
    reader: &mut Reader<BufReader<File>>
) -> Result<String, quick_xml::Error> {
    let mut element_buf = Vec::new();
    let _event = reader.read_event_into(&mut element_buf)?;
    let s = String::from_utf8(element_buf)?;
    //println!("Found filename[{}]",s.trim());
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
    //println!("Found [{}]",s.trim());
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
    let mut dc_attrs : HashMap<String, String> = HashMap::new();
    loop {
        let event = reader.read_event_into(&mut buf)?;
        let evt2 = event.clone();
        match evt2 {
            Event::Start(elt) => {
                if elt.name().as_ref().starts_with("dc:".as_ref()) {
                    let info = String::from_utf8_lossy(elt.name().as_ref()).to_string();
                    //println!("Found dc: elt => {}",info);
                    add_dc(&mut reader, &mut dc_attrs,info);
                }
            },
            Event::Eof => break,
            _ =>(),
        }
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
                    save_filename(&mut reader)?;
                    // TODO compute and check hash of file
                },
                // TODO extract sum-up attributes
                b"sum-up" => {
                    println!("sum-up found");
                    save_sum_up(&mut reader, element)?;
                }
                // TODO extract dc:xx fields from descriptive-metadata
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