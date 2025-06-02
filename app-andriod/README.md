
# HeartIO Android App

This Android app receives Bluetooth heart rate broadcasts from wearable devices and forwards them to your computer. This is required when using devices that limit Bluetooth broadcasts to the currently connected phone.

Enable `APPLE_WATCH: true` in `heartio.config.json` on your computer - when this option is enabled, the computer HeartIO software will receive heart rate data from the mobile app instead of connecting directly to Bluetooth devices.

## Xiaomi Band 6/7 Setup Guide

Xiaomi bands restrict Bluetooth heart rate broadcasts to only the currently connected phone, requiring an Android app to forward data to your computer. iOS forwarding is not currently supported.

### Setup Steps

1. **Install APK**: Download and install the APK from [GitHub Releases](https://github.com/xn-sakina/vrchat-heartio/releases)

2. **Configure**: Set `APPLE_WATCH: true` in `heartio.config.json` to receive forwarded heart rate data (this option also works for [Apple Watch](../app-apple-watch/README.md))

3. **Enable Xiaomi Band**: Enable the following settings on your band:
   - Bluetooth broadcast
   - Heart rate broadcast
   - GPS
   - Workout GO

The app will detect heart rate data and forward it to your computer.
