use core::str::FromStr;
use esp_idf_sys::esp_random;

pub fn get_random_string(len: usize) -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                    abcdefghijklmnopqrstuvwxyz\
                    0123456789";
    let mut result = String::with_capacity(len);
    for _ in 0..len {
        let rnd = unsafe { esp_random() };
        let idx = (rnd % (charset.len() as u32)) as usize;
        result.push(charset[idx] as char);
    }
    result
}

pub fn to_heapless<const N: usize>(s: &str) -> anyhow::Result<heapless::String<N>> {
    heapless::String::<N>::from_str(s).map_err(|e| anyhow::anyhow!("{:?}", e))
}
