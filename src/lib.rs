mod file_info;

use wasm_bindgen::prelude::*;
use std::io::{BufReader};
use calamine::{Reader, DataType, Xlsx, open_workbook_from_rs};
use std::io::Cursor;
use std::io::Seek;
use std::io::Read;
use web_sys::{Worker, FileReader, CustomEvent, CustomEventInit, window};
use js_sys::{Uint8Array};
use serde::{Serialize, Deserialize};
use gloo_utils::format::JsValueSerdeExt;
use chrono::{ NaiveDateTime, NaiveDate};
use file_info::{FileInfo};

static mut a:i32 = 10;

fn send_event(){
    let event = CustomEvent::new_with_event_init_dict(
    "clear_invoice_update",
    CustomEventInit::new()
        .bubbles(true)
        .cancelable(true),
    ).unwrap();
    let document = window().unwrap().document().expect("expecting a document on window");
    document
    .dispatch_event(&event);
}


#[wasm_bindgen]
pub fn to_get()-> JsValue {
    unsafe {
        return serde_wasm_bindgen::to_value(&a).unwrap();
    }
}

#[derive(Serialize, Deserialize)]
pub enum MovementEntry {
    CREDIT,
    DEBIT,
    UNDEFINED
}

#[derive(Serialize, Deserialize)]
pub enum MovementOperation {
    REDEEM,
    YIELD,
    EQUITY_INTEREST,
    LIQUIDATION,
    DIVIDEND,
    UNDEFINED,
    UNDEFINED_FIXED_INCOME
}


#[derive(Serialize, Deserialize)]
pub struct Movement {
    pub row: i16,
    pub id: String,
    pub alias: String,
    pub entry: MovementEntry,
    pub date: i64,
    pub operation: MovementOperation,
    pub product: String,
    pub holder: String,
    pub quantity: f64,
    pub price_unit: f64,
    pub price_total:f64,
    pub origin: String
}

#[wasm_bindgen]
pub fn is_movement_file(uint8:Uint8Array)-> bool {
    let array = Uint8Array::new(&uint8);
    let bytes: Vec<u8> = array.to_vec();
    let c = Cursor::new(bytes);
    let mut excel: Xlsx<_> = match open_workbook_from_rs(c){
        Ok(file)=>file,
        Err(e)=> return false
    };
    if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        // Entrada/Saída	Data	Movimentação	Produto	Instituição	Quantidade	Preço unitário	Valor da Operação
        for row in r.rows() {
            let movement_type = match row[0].get_string(){
                Some(str)=>str.to_string(),
                None => return false
            };
            if movement_type.contains("Entrada") {  
                return true;
            } else{
                return false;
            }
        }
        return false;
    } else{
        return false;
    }
}

#[wasm_bindgen]
pub fn process_movement_file(fileName:String, file:FileReader)-> JsValue {
    let result = file.result().unwrap();
    let array = Uint8Array::new(&result);
    let bytes: Vec<u8> = array.to_vec();
    let c = Cursor::new(bytes);
    let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
    
    let mut movements = vec![];
    if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        let mut i = 0;
        for row in r.rows() {
            let movement_type = row[0].get_string().unwrap().to_string();
            if !movement_type.contains("Entrada") {                
                let mut m = Movement{
                    row: i,
                    id: format!("{}{}{}",fileName.to_string(),"_",i.to_string()),
                    alias: "".to_string(),
                    entry: MovementEntry::UNDEFINED,
                    date: 0,
                    operation: MovementOperation::UNDEFINED,
                    product: "".to_string(),
                    holder: "".to_string(),
                    quantity: 0.0,
                    price_unit: 0.0,
                    price_total: 0.0,
                    origin: fileName.to_string()
                };
                i = i + 1;
                row_to_movement(row, &mut m);
                movements.push(m);
            }
        }
    }
    send_event();
    return serde_wasm_bindgen::to_value(&movements).unwrap();
}

#[derive(Debug,Serialize, Deserialize)]
pub struct MovementFileInfoResponse{
    pub event:String,
    pub data:FileInfo,
}
#[wasm_bindgen]
pub fn get_moviment_file_info_using_event(worker:Worker, id:i32, uint8:Uint8Array) {
    let mut file_info = FileInfo{
        numOfRows:0,
        name:"".to_string(),
        id:id
    };
    let array = Uint8Array::new(&uint8);
    let bytes: Vec<u8> = array.to_vec();
    let c = Cursor::new(bytes);
    let mut excel: Xlsx<_> = match open_workbook_from_rs(c){
        Ok(file)=>file,
        Err(e)=> panic!("{}",e)
    };
    get_info_from_excel(&mut file_info, &mut excel);

    let response = MovementFileInfoResponse{
        event: "MOVEMENT_FILE_INFO".to_string(),
        data:file_info
    };
    let value = serde_wasm_bindgen::to_value(&response).unwrap();
    worker.post_message(&value);
}

fn row_to_movement(row:&[DataType], m:&mut Movement){
    match row[0].get_string(){
        Some("Credito") => m.entry = MovementEntry::CREDIT,
        Some("Debito") => m.entry = MovementEntry::DEBIT,
        _ => m.entry = MovementEntry::UNDEFINED
    };
    let naive_date = NaiveDate::parse_from_str(row[1].get_string().unwrap(), "%d/%m/%Y").unwrap();
    let naive_datetime = naive_date.and_hms(12,0,0);
    m.date = naive_datetime.timestamp_millis();
   
    match row[2].get_string(){
        Some("Transferência - Liquidação") => m.operation = MovementOperation::LIQUIDATION,
        Some("Rendimento") => m.operation = MovementOperation::YIELD,
        Some("Dividendo") => m.operation = MovementOperation::DIVIDEND,
        Some("Juros Sobre Capital Próprio") => m.operation = MovementOperation::EQUITY_INTEREST,
        Some("COMPRA / VENDA") => m.operation = MovementOperation::UNDEFINED_FIXED_INCOME,
        Some("VENCIMENTO") => m.operation = MovementOperation::UNDEFINED_FIXED_INCOME,
        _ => m.operation = MovementOperation::UNDEFINED
    };
    let product = row[3].get_string().unwrap();
    let splited:Vec<&str> = product.split(" - ").collect();
    m.product = product.to_string();
    m.alias = splited[0].to_string();
    m.holder = row[4].get_string().unwrap().to_string();
    m.quantity = row[5].get_float().unwrap();
    match row[6].get_float(){
        Some(f) => m.price_unit = f,
        None =>{}
    };
    match row[7].get_float(){
        Some(f) => m.price_total = f,
        None =>{}
    };
}

pub fn get_info_from_excel<RS>(file_info:&mut FileInfo, excel:&mut Xlsx<RS>)  where RS: Seek, RS: std::io::Read{
    if let Some(Ok(res)) = excel.worksheet_range("Movimentação") {
        let rows = res.rows();
        file_info.numOfRows = rows.len() as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
 
    #[test]
    fn asdf() {
        let data = fs::read("assets/movimentacao-2022-01.xlsx").unwrap();
        let c = Cursor::new(data);
        let mut excel: Xlsx<_> = open_workbook_from_rs(c).unwrap();
        let mut owned_string: String = "hello ".to_owned();

        if let Some(Ok(r)) = excel.worksheet_range("Movimentação") {
        for row in r.rows() {
            let movement_type = row[0].get_string().unwrap().to_string();
            if !movement_type.contains("Entrada") {                
                let mut m = Movement{
                    entry: MovementEntry::UNDEFINED,
                    date: 0,
                    operation: MovementOperation::UNDEFINED,
                    product: "".to_string(),
                    holder: "".to_string(),
                    quantity: 0.0,
                    price_unit: 0.0,
                    price_total: 0.0,
                    origin: "".to_string()
                };
                row_to_movement(row, &mut m);
            }
            println!("a{:?}", row);
             }
    }
        println!("a{:?}", owned_string);
        assert_eq!("excel", "Hello from rust, f");
    }
}
