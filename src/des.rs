//! Deserializers for the core types.

use serde::{Deserialize, Deserializer};

pub fn des_null_to_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let v = Option::<T>::deserialize(de)?;
    Ok(v.unwrap_or_default())
}
