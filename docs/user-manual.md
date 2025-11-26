# User Manual

## Initial Configuration

When first powered on, the device starts in Access Point mode.
- It displays the network name and password on the screen. Connect to that network with your computer.
- Open your browser and navigate to https://retro-clock.local - you will be logged in automatically.
- Download the root CA certificate from the blue box on the **Access** tab.
- The certificate checksum will be shown on the display. Compare it against the checksum of your downloaded file using `crc32 rootCA.pem`.
- Set up an admin password. This will be used later for logging into the web UI.
- Set up the details of your home network.
- Save changes and confirm when prompted to restart your device. Alternatively, you can switch the power off and on again.
- The device will boot up and connect to your home network. If anything goes wrong at this point, the device will switch back to Access Point mode and the configuration can be corrected.

Note: `crc32` can be found for instance in [libarchive](https://packages.debian.org/sid/libarchive-tools) package.

## Runtime Configuration

After completing the initial configuration, the device connects to your home network. The web UI is available at https://retro-clock.local as in Access Point mode. This time it requires the admin password for logging in. If you forget your password, see the next section. The following settings can be modified:

<table>

<tr>
<td>

**System Tab**

</td>
<td>

Coordinates and timezone can be set manually when you are connected to a mobile network. In this scenario, the device may incorrectly determine its location.
Additionally, you may want to manually set coordinates if you would like the device to show the weather forecast for a remote location.

</td>
</tr>

<tr>
<td>

**World Time Tab**

</td>
<td>

Allows you to select timezones for remote locations of interest. Timezones can have aliases assigned, for example: "Work = Europe/Dublin".

</td>
</tr>

<tr>
<td>

**Weather Tab**

</td>
<td>

Allows you to select weather stations and assign them names. For a list of available providers and supported URLs, visit https://weather.krason.dev/ and navigate to Settings → Edit.

</td>
</tr>

</table>


## Resetting Admin Password

To change the admin password:
- Restart the device (using the power switch) and press the button immediately after turning the power on.
- Keep holding the button until the device starts in Access Point mode and displays the network details and password.
- Continue as described in Initial Configuration.

## Resetting Network Credentials

If you change the credentials of your home network, the retro-clock will not be able to connect anymore. Simply restart your device using the power switch. It will boot up in Access Point mode. Continue as described in Initial Configuration.

## Power-Safe Mode

For full functionality, the retro-clock should be powered by a USB-C cable. If power is lost, it will switch to battery power.
It can operate like this for several days, but its functionality is limited to displaying the time only. The button and ambient light are disabled.
In this mode, time can drift by several minutes per day.
Immediately after external power is restored, the retro-clock will reconnect to your home network (as soon as it becomes available) and immediately resynchronize with the NTP server.

## Startup and Time Synchronization

For normal operation, the retro-clock must have Initial Configuration completed. It can boot up with or without external power connected. In either case, it needs to connect to the home network, detect its location, and synchronize with an NTP server.
The device synchronizes with an NTP server once per hour. Time correction is applied immediately. If the network connection is lost, the device will automatically keep reconnecting until it is restored.

## Using the Button

The button on the top of the device toggles between World Time, Forecast, and Weather pages.

- Hold the button to switch pages
- Press the button to switch sub-pages
- After 60 seconds of inactivity, the retro-clock will return to the first page and sub-page (Current Time)

### World Time Page

The first page shows the local date and time. `Wxy` on the display denotes the week number.
The following sub-pages show time in remote locations, as configured in the Web UI.

### Forecast Page

The first line shows the forecast for the current day (today). The second line shows the forecast for the following day (tomorrow).

Precipitation should be read as:
- Maximum probability of precipitation [%]
- Duration of precipitation [hours]
- Total precipitation [mm]

Cloud data should be read as:
- Mean cloud coverage
- Absolute deviation of cloud coverage
- Maximum UV index

### Weather Page

Weather data should be read as follows:
- Time of the reading [minutes ago]
- Humidity [%]
- Precipitation [mm]
- Wind direction
- Wind mean speed
- Wind gusts or maximum speed of the day (depending on the provider)

## Busy & Error Indicators

- Green LED indicates that the retro-clock is busy downloading data.
- Red LED after downloading means that some errors occurred. Try again to clear the error.
- Red LED can also indicate that the connection to the network has been lost. It will be cleared automatically after the connection is restored. This is normal when external power becomes unavailable and then becomes available again.