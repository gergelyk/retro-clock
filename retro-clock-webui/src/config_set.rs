#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum ConfigSet {
    WifiSsid,
    SystemSettings,
    WorldTime,
    Weather,
}
