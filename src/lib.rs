use wasm_bindgen::prelude::*;
use std::io::{BufReader};
use calamine::{Reader, DataType, Xlsx, open_workbook_from_rs};
use std::io::Cursor;
use web_sys::{FileReader};
use js_sys::{Uint8Array};
use serde::{Serialize, Deserialize};
use gloo_utils::format::JsValueSerdeExt;
use chrono::{ NaiveDateTime, NaiveDate};

static mut a:i32 = 10;

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
    return serde_wasm_bindgen::to_value(&movements).unwrap();
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
