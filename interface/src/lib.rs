use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConverterError {
    #[error("Error converting bytes: {0}")]
    BytesConvertError(String),
}

#[derive(Debug, PartialEq)]
pub struct NetworkPacket {
    pub version: String,
    pub units: String,
    pub location: String,
    pub data: Vec<f32>,
}

pub trait UdpAble {
    type Item;
    fn to_bytes(self) -> Result<Vec<u8>, ConverterError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self::Item, ConverterError>;
}

pub const BUFFER_SIZE: usize = 1024;

impl Default for NetworkPacket {
    fn default() -> Self {
        Self {
            version: "0.0".to_string(),
            units: "".to_string(),
            location: "".to_string(),
            data: vec![],
        }
    }
}

fn write_into_buffer<'a>(
    bytes: &'a mut Vec<u8>,
    source_slice: &'a [u8],
    start: usize,
    length: Option<usize>,
) -> &'a Vec<u8> {
    let length = match length {
        Some(v) => v,
        None => bytes.len() - start,
    };

    let end = start + length;
    let bytes_to_copy = std::cmp::min(length, source_slice.len());

    bytes[start..start + bytes_to_copy].copy_from_slice(source_slice);

    if bytes_to_copy < length {
        bytes[start + bytes_to_copy..end].fill(0);
    }

    bytes
}

impl UdpAble for NetworkPacket {
    type Item = Self;

    fn to_bytes(self) -> Result<Vec<u8>, ConverterError> {
        match self.version.as_str() {
            "0.0" => {
                let mut bytes = vec![0u8; BUFFER_SIZE];
                let mut version_iter = self.version.bytes();
                // encode version in first two bytes
                bytes[0] = version_iter.next().unwrap();
                version_iter.next();
                bytes[1] = version_iter.next().unwrap();

                // allow the next 10 bytes for units
                write_into_buffer(&mut bytes, self.units.as_bytes(), 2, Some(10));

                // write the location in
                write_into_buffer(&mut bytes, self.location.as_bytes(), 12, Some(52));

                write_into_buffer(&mut bytes, f32_vec_to_u8_vec(&self.data), 64, None);

                return Ok(bytes);
            }
            _ => {
                return Err(ConverterError::BytesConvertError(
                    "version not existing".to_string(),
                ));
            }
        };
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self::Item, ConverterError> {
        let major = bytes[0] as char;
        let minor = bytes[1] as char;

        match (major, minor) {
            ('0', '0') => {
                let version = format!("{}.{}", major, minor);
                let units = String::from_utf8(bytes[2..12].to_vec()).unwrap();
                let location = String::from_utf8(bytes[12..64].to_vec()).unwrap();
                let data = u8_to_f32_vec(&bytes[64..]);
                return Ok(Self::Item {
                    version,
                    units,
                    location,
                    data,
                });
            }
            _ => {
                return Err(ConverterError::BytesConvertError(format!(
                    "version not existing: {}.{}",
                    major, minor
                )));
            }
        };
    }
}
fn u8_to_f32_vec(v: &[u8]) -> Vec<f32> {
    v.chunks_exact(4)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .map(f32::from_le_bytes)
        .collect()
}

fn f32_vec_to_u8_vec(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_thing(data: &Vec<f32>) -> NetworkPacket {
        let np = NetworkPacket {
            data: data.clone(),
            ..Default::default()
        };
        np
    }

    #[test]
    fn test_to_bytes() {
        let data = vec![3., 4.];
        let np = create_thing(&data);
        let bytes = np.to_bytes().unwrap();
        // println!("{:?}", bytes);
        // println!("{:?}", &bytes[64..66]);
        assert_eq!(vec![0, 0, 64, 64, 0, 0, 128, 64], &bytes[64..64+8]);
    }

    #[test]
    fn test_from_bytes() {
        let data = vec![3., 4.];
        let np = create_thing(&data);
        let bytes = np.to_bytes().unwrap();
        let parsed = NetworkPacket::from_bytes(&bytes).unwrap();
        assert_eq!(data, &parsed.data[0..2])
    }
}
