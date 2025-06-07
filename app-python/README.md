# HeartIO Python Reporter

This sub-project serves as a broadcaster/reporter that forwards heart rate broadcast data from Xiaomi bands (generation 6 & 7) to the main HeartIO program.

## Purpose

Since Xiaomi bands 6 & 7 do not support international Bluetooth heart rate standards, we cannot directly subscribe to their heart rate data. This broadcaster works as a workaround by:

1. Receiving broadcast data from Xiaomi bands 6 & 7
2. Forwarding the heart rate data to the main HeartIO program
3. The main program then sends the data to OSC

## Limitations

This solution has some limitations in speed:
- BPM data reception: every `2-5` seconds

## Usage

1. Download the reporter from [GitHub Releases](https://github.com/xn-sakina/vrchat-heartio/releases)
2. Enable `APPLE_WATCH: true` in your `heartio.config.json`
3. Start both the reporter and HeartIO main program

## Recommendation

This is a workaround solution. For optimal performance, we recommend purchasing devices that support international Bluetooth heart rate standards.
