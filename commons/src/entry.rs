// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

pub trait Key: 'static + Send + Sync {
    fn collect(&self) -> impl AsRef<[u8]>;

    fn into_key(value: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

pub trait Value:
    'static + Clone + PartialEq + Send + Sync + Encode<Output = Bytes> + Decode<Input = [u8]>
{
}

impl Key for u32 {
    fn collect(&self) -> impl AsRef<[u8]> {
        self.to_be_bytes()
    }

    fn into_key(value: &[u8]) -> Result<Self> {
        Ok(u32::from_be_bytes(<[u8; 4]>::try_from(value)?))
    }
}

impl Key for u64 {
    fn collect(&self) -> impl AsRef<[u8]> {
        self.to_be_bytes()
    }

    fn into_key(value: &[u8]) -> Result<Self> {
        Ok(u64::from_be_bytes(<[u8; 8]>::try_from(value)?))
    }
}

impl<const N: usize> Key for [u8; N] {
    fn collect(&self) -> impl AsRef<[u8]> {
        self.as_slice()
    }

    fn into_key(value: &[u8]) -> Result<Self> {
        if value.len() != N {
            bail!(
                "Invalid size for [u8; {}]: expected {} bytes, got {}",
                N,
                N,
                value.len()
            );
        }

        let mut array = [0u8; N];
        array.copy_from_slice(value);
        Ok(array)
    }
}

impl<T: 'static + Clone + PartialEq + Send + Sync + Encode<Output = Bytes> + Decode<Input = [u8]>>
    Value for T
{
}
