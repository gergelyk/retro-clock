// There are crates providing similar functionality, but I couldn't find any that would:
// - accept consts as parameters
// - limit vectors
// - support Option<String>>

use serde::de;
use serde::{Deserialize, Deserializer};
pub struct StringLenLimiter<const N: usize>;

impl<const N: usize> StringLenLimiter<N> {
    fn check_len<E: de::Error>(s: String) -> Result<String, E> {
        if s.len() > N {
            Err(E::custom(format!(
                "string too long: len={}, max_len={}",
                s.len(),
                N
            )))
        } else {
            Ok(s)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::check_len(s)
    }

    pub fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => Ok(Some(Self::check_len(s)?)),
            None => Ok(None),
        }
    }
}

pub struct VecLenLimiter<const N: usize, T>(pub Vec<T>);

impl<'de, const N: usize, T> VecLenLimiter<N, T>
where
    T: Deserialize<'de>,
{
    fn check_len<E: de::Error>(v: Vec<T>) -> Result<Vec<T>, E> {
        if v.len() > N {
            Err(E::custom(format!(
                "vector too long: len={}, max_len={}",
                v.len(),
                N
            )))
        } else {
            Ok(v)
        }
    }
}

impl<'de, const N: usize, T> Deserialize<'de> for VecLenLimiter<N, T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        let v = VecLenLimiter::<N, T>::check_len(v)?;
        Ok(VecLenLimiter(v))
    }
}
