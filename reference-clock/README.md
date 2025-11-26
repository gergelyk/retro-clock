# Reference Clock

Displays system time. The goal is to refresh shortly (<1ms) after each round second.

## Setup

- Set NTP server in /etc/systemd/timesyncd.conf to the same as used in retro-clock-esp: pool.ntp.org
- Enforce NTP (re)synchronization:
    ```sh
    sudo systemctl restart systemd-timesyncd
    ```
- Check synchronization status:
    ```sh
    timedatectl status
    timedatectl timesync-status
    ```

## Usage

```sh
cargo run
```
