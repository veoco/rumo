use super::forms::FieldCreate;
use crate::common::errors::FieldError;

pub fn get_field_params(
    field_create: &FieldCreate,
) -> Result<(String, Option<String>, i32, f32), FieldError> {
    let str_value;
    let int_value;
    let float_value;
    let field_type = match field_create.r#type.as_str() {
        "str" => {
            let value = field_create.str_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams("type and str_value".to_string()));
            }
            str_value = Some(value.unwrap());
            int_value = 0;
            float_value = 0f32;
            "str"
        }
        "int" => {
            let value = field_create.int_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams("type and int_value".to_string()));
            }
            str_value = None;
            int_value = value.unwrap();
            float_value = 0f32;
            "int"
        }
        "float" => {
            let value = field_create.float_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams(
                    "type and float_value".to_string(),
                ));
            }
            str_value = None;
            int_value = 0;
            float_value = value.unwrap();
            "float"
        }
        _ => return Err(FieldError::InvalidParams("type".to_string())),
    };
    Ok((field_type.to_string(), str_value, int_value, float_value))
}