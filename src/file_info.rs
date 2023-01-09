use serde::{Serialize, Deserialize};

#[derive(Debug,Serialize, Deserialize)]
pub struct FileInfo{
    pub numOfRows:i32,
    pub name:String,
    pub id:i32
}
