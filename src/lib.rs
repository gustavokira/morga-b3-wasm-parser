use wasm_bindgen::prelude::*;
use std::io::{BufReader};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use calamine::{Reader, Xlsx, open_workbook_from_rs, open_workbook};
use std::io::Cursor;
use web_sys::{File, FileReader, console};
use js_sys::{Uint8Array};
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn yo(file:FileReader)-> JsValue {
    let result = file.result().unwrap();
    let array = Uint8Array::new(&result);
    let bytes: Vec<u8> = array.to_vec();
    let c = Cursor::new(bytes);
    let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
    
    let mut owned_string: String = "hello ".to_owned();

    if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        for row in r.rows() {
            owned_string.push_str(&row[0].get_string().unwrap());
            //println!("a{:?}", row[0]);
            //println!("row={:?}", row);
        }
    }
    return  JsValue::from_str(&owned_string);
}

#[wasm_bindgen]
pub fn add(i:&str) -> JsValue {
    let c = Cursor::new(i.as_bytes().to_vec());
   // let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
   // let mut x = JsValue::from_str("d");
   // if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        // for row in r.rows() {
            //x = JsValue::from_str(&format!("a{:?}", row[0]));
            //println!("a{:?}", row[0]);
            //println!("row={:?}", row);
            //return JsValue::from_str(&format!("a{:?}", row[0]));
        // }
    //}
    return JsValue::from_str("axd");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::fs;
    use std::io::{BufReader, BufRead, Read};
    use encoding_rs::WINDOWS_1252;
    use encoding_rs_io::DecodeReaderBytesBuilder;
    #[test]
    // fn a() {
    //     let file = File::open("assets/movimentacao-2022-01.xlsx").unwrap();
    //     let mut reader = BufReader::new(
    //         DecodeReaderBytesBuilder::new()
    //         .encoding(Some(WINDOWS_1252))
    //             .build(file));

    //             let mut str = String::new();
    //             reader.read_to_string(&mut str);

    //             let c = Cursor::new(str.as_bytes().to_vec());
    // let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
    //  if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
    //     for row in r.rows() {
    //          println!("a{:?}", row[0]);
    //          println!("row={:?}", row);
    //          }
    // }
    //     assert_eq!("excel", "Hello from rust, f");
    // }
    // #[test]
    // fn it_works() {
    //     let file = File::open("assets/movimentacao-2022-01.xlsx").unwrap();
    //     let mut reader = BufReader::new(
    //         DecodeReaderBytesBuilder::new()
    //             .build(file));



    //             let mut dest = vec![];
    //             reader.read_to_end(&mut dest).unwrap();
    //             let c = Cursor::new(dest);
    // let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
    //  if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
    //     for row in r.rows() {
    //          println!("a{:?}", row[0]);
    //          println!("row={:?}", row);
    //          }
    // }
    //     assert_eq!("excel", "Hello from rust, f");
    // }
    fn asdf() {
        let data = fs::read("assets/movimentacao-2022-01.xlsx").unwrap();
        let x = data.to_vec();
        println!("{:?}",x);
        let c = Cursor::new(data);
    let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
     if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        for row in r.rows() {
             println!("a{:?}", row[0]);
             println!("row={:?}", row);
             }
    }
        assert_eq!("excel", "Hello from rust, f");
    }
}
