use core::convert::TryInto;
use exn::ResultExt;
use postcard;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg(feature = "lz4")]
use lz4_flex;
#[cfg(feature = "flate")]
use flate2::read::GzDecoder;
#[cfg(feature = "flate")]
use flate2::write::GzEncoder;
#[cfg(feature = "flate")]
use flate2::Compression;
#[cfg(feature = "zstd")]
use zstd;

use crc32fast::hash as crc32_hash;

use crate::error;

pub type BazaR<T> = Result<T, exn::Exn<error::Error>>;

const MAGIC: &[u8; 4] = b"BZA1";
const VERSION: u8 = 1;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    None = 0,
    Lz4 = 1,
    Deflate = 2,
    Zstd = 3,
}

impl From<u8> for Algorithm {
    fn from(v: u8) -> Self {
        match v {
            1 => Algorithm::Lz4,
            2 => Algorithm::Deflate,
            3 => Algorithm::Zstd,
            _ => Algorithm::None,
        }
    }
}

/// Encode a serializable value into the custom dump format.
pub fn dump<T: Serialize>(value: &T, alg: Algorithm) -> BazaR<Vec<u8>> {
    // Serialize with postcard (compact)
    let serialized = postcard::to_stdvec(value)
        .map_err(|e| exn::Exn::new(error::Error::Message(format!("Serialize error: {}", e))))?;

    let uncompressed_len = serialized.len() as u64;
    let checksum = crc32_hash(&serialized);

    // Compress according to algorithm
    let payload: Vec<u8> = match alg {
        Algorithm::None => serialized.clone(),
        Algorithm::Lz4 => {
            #[cfg(feature = "lz4")]
            {
                lz4_flex::compress(&serialized)
            }
            #[cfg(not(feature = "lz4"))]
            {
                return Err(exn::Exn::new(error::Error::Message(
                    "LZ4 feature not enabled".into(),
                )));
            }
        }
        Algorithm::Deflate => {
            #[cfg(feature = "flate")]
            {
                use std::io::Write;
                let mut e = GzEncoder::new(Vec::new(), Compression::default());
                e.write_all(&serialized)
                    .map_err(|e| exn::Exn::new(e.into()))?;
                e.finish().map_err(|e| exn::Exn::new(e.into()))?
            }
            #[cfg(not(feature = "flate"))]
            {
                return Err(exn::Exn::new(error::Error::Message(
                    "Deflate feature not enabled".into(),
                )));
            }
        }
        Algorithm::Zstd => {
            #[cfg(feature = "zstd")]
            {
                zstd::stream::encode_all(&*serialized, 3).map_err(|e| exn::Exn::new(e.into()))?
            }
            #[cfg(not(feature = "zstd"))]
            {
                return Err(exn::Exn::new(error::Error::Message(
                    "Zstd feature not enabled".into(),
                )));
            }
        }
    };

    // Build header
    let mut out = Vec::with_capacity(32 + payload.len());
    out.extend_from_slice(MAGIC); // 4
    out.push(VERSION); // 1
    out.push(alg as u8); // 1
    out.push(0); // reserved
    out.extend_from_slice(&uncompressed_len.to_le_bytes()); // 8
    out.extend_from_slice(&checksum.to_le_bytes()); // 4
    out.extend_from_slice(&payload);

    Ok(out)
}

/// Restore a value from bytes produced by `dump`.
pub fn restore<T: DeserializeOwned>(data: &[u8]) -> BazaR<T> {
    // minimal header length: 4+1+1+1+8+4 = 19
    if data.len() < 19 {
        exn::bail!(error::Error::Message("Input too short for dump header".into()));
    }

    if &data[0..4] != MAGIC {
        exn::bail!(error::Error::Message("Invalid magic header".into()));
    }

    let ver = data[4];
    if ver != VERSION {
        exn::bail!(error::Error::Message("Unsupported dump version".into()));
    }

    let alg = Algorithm::from(data[5]);
    // data[6] reserved

    let uncompressed_len = u64::from_le_bytes(
        data[7..15]
            .try_into()
            .map_err(|_| exn::Exn::new(crate::error::Error::Message("Invalid header (len)".into())))?,
    );
    let checksum = u32::from_le_bytes(
        data[15..19]
            .try_into()
            .map_err(|_| exn::Exn::new(crate::error::Error::Message("Invalid header (checksum)".into())))?,
    );

    let payload = &data[19..];

    let decompressed: Vec<u8> = match alg {
        Algorithm::None => payload.to_vec(),
        Algorithm::Lz4 => {
            #[cfg(feature = "lz4")]
            {
                // Try size-prepended variant first, then generic decompress
                if let Ok(v) = lz4_flex::decompress_size_prepended(payload) {
                    v
                } else {
                    lz4_flex::decompress(payload, uncompressed_len as usize)
                        .map_err(|e| exn::Exn::new(error::Error::Message(format!("LZ4 decompress error: {}", e))))?
                }
            }
            #[cfg(not(feature = "lz4"))]
            {
                exn::bail!(error::Error::Message("LZ4 feature not enabled".into()));
            }
        }
        Algorithm::Deflate => {
            #[cfg(feature = "flate")]
            {
                use std::io::Read;
                let mut d = GzDecoder::new(payload);
                let mut out = Vec::with_capacity(uncompressed_len as usize);
                d.read_to_end(&mut out).map_err(|e| exn::Exn::new(e.into()))?;
                out
            }
            #[cfg(not(feature = "flate"))]
            {
                exn::bail!(error::Error::Message("Deflate feature not enabled".into()));
            }
        }
        Algorithm::Zstd => {
            #[cfg(feature = "zstd")]
            {
                zstd::stream::decode_all(payload).map_err(|e| exn::Exn::new(e.into()))?
            }
            #[cfg(not(feature = "zstd"))]
            {
                exn::bail!(error::Error::Message("Zstd feature not enabled".into()));
            }
        }
    };

    if decompressed.len() != uncompressed_len as usize {
        exn::bail!(error::Error::Message("Uncompressed length mismatch".into()));
    }

    let sum = crc32_hash(&decompressed);
    if sum != checksum {
        exn::bail!(error::Error::Message("Checksum mismatch".into()));
    }

    let v: T = postcard::from_bytes(&decompressed)
        .map_err(|e| exn::Exn::new(error::Error::Message(format!("Deserialize error: {}", e))))?;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct S {
        a: u32,
        b: String,
    }

    #[test]
    fn roundtrip_none() {
        let s = S { a: 42, b: "hello".into() };
        let dumped = match dump(&s, Algorithm::None) {
            Ok(d) => d,
            Err(e) => panic!("dump failed: {}", e),
        };
        let restored: S = match restore(&dumped) {
            Ok(r) => r,
            Err(e) => panic!("restore failed: {}", e),
        };
        assert_eq!(restored, s);
    }

    #[test]
    fn roundtrip_lz4() {
        let s = S { a: 123, b: "some longer text to benefit from compression".into() };
        #[cfg(feature = "lz4")]
        {
            let dumped = match dump(&s, Algorithm::Lz4) {
                Ok(d) => d,
                Err(e) => panic!("dump failed: {}", e),
            };
            let restored: S = match restore(&dumped) {
                Ok(r) => r,
                Err(e) => panic!("restore failed: {}", e),
            };
            assert_eq!(restored, s);
        }
    }
}
