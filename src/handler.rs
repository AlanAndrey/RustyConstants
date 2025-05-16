// here is the quasi backend
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ConstantRequestPayload {
    pub names: Vec<String>,
}
#[derive(Serialize, Debug)]
pub struct ConstantProcessResult {
    pub name: String,
    pub value: f64,
    pub unit: String,
}


pub fn process_constant(name :&str) -> ConstantProcessResult {
    // Simulate some processing
    let result = ConstantProcessResult {
        name: "test".to_string(),
        value: 0.5,
        unit: "unit".to_string(),
    };

    return result;
}