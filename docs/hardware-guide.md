# Hardware Guide

The prototype has a 3-state power switch:
- **BTT**: Power from the battery module. The only officially supported power source.
- **DBG**: Power from the debugging port (USB-C). Used while programming.
- **USB**: Power directly from USB-B. Bypasses the battery module and DC-DC converter. Only for prototyping.

Before programming:
- Disconnect the USB-B cable (if connected)
- Switch the power switch to **DBG** position
- Connect your computer to the debugging port (USB-C)

## Troubleshooting

If you encounter problems during programming, you can force the bootloader to load by holding the button while powering on the device.