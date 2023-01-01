use serde::{ser, Serialize};

use super::errors::{Error, Result};

pub struct Serializer {
    output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &(String::from("i:") + &v.to_string());
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        unimplemented!()
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        unimplemented!()
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += &format!("s:{}:\"", v.len());
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<()> {
        unimplemented!()
    }

    fn serialize_some<T>(self, _v: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<()> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += &format!("a:{}:{{", len.unwrap_or(0));
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ";";
        }
        key.serialize(&mut **self)?;
        self.output += ";";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += ";}";
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        unimplemented!()
    }
}

#[test]
fn test_to_string() {
    use super::models::AttachmentText;

    let s = AttachmentText {
        name: "mate10-emui-1.png".to_string(),
        path: "/usr/uploads/2022/12/3448342241.png".to_string(),
        size: 14660,
        r#type: "png".to_string(),
        mime: "image/png".to_string(),
    };
    let expected = r#"a:5:{s:4:"name";s:17:"mate10-emui-1.png";s:4:"path";s:35:"/usr/uploads/2022/12/3448342241.png";s:4:"size";i:14660;s:4:"type";s:3:"png";s:4:"mime";s:9:"image/png";}"#;
    assert_eq!(to_string(&s).unwrap(), expected);

    let s = AttachmentText {
        name: "https-get-status-1.png".to_string(),
        path: "/usr/uploads/2022/12/2415673155.png".to_string(),
        size: 25523,
        r#type: "png".to_string(),
        mime: "image/png".to_string(),
    };
    let expected = r#"a:5:{s:4:"name";s:22:"https-get-status-1.png";s:4:"path";s:35:"/usr/uploads/2022/12/2415673155.png";s:4:"size";i:25523;s:4:"type";s:3:"png";s:4:"mime";s:9:"image/png";}"#;
    assert_eq!(to_string(&s).unwrap(), expected);
}
