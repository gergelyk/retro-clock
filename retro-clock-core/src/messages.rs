pub enum BootupMessage {
    ConnectingToWiFi,
    ObtainingIp,
    SynchronizingWithNtp,
    ObtainingLocationInfo,
    ObtainingUtcOffset,
}

impl BootupMessage {
    pub fn as_str(&self) -> &'static str {
        match self {
            BootupMessage::ConnectingToWiFi => "Connecting to\nWiFi…",
            BootupMessage::ObtainingIp => "Obtaining an IP\naddress…",
            BootupMessage::SynchronizingWithNtp => "Synchronizing\nwith NTP server…",
            BootupMessage::ObtainingLocationInfo => "Obtaining\nlocation info…",
            BootupMessage::ObtainingUtcOffset => "Obtaining an\noffset to UTC…",
        }
    }
}

impl AsRef<str> for BootupMessage {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
