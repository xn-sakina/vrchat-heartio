# vrchat-heartio

A real-time heart rate monitoring solution that connects to Bluetooth heart rate devices and displays your heart rate in VRChat through OSC messages.

## Features

- Connect directly to Bluetooth heart rate devices (`Polar`, `CooSpo`, `Garmin`, etc.)
- Automatically displays heart rate in VRChat via OSC protocol
- Dynamic heart display styles based on BPM levels
- Data logging with SQLite for tracking your heart rate over time

## Setup

1. Install dependencies:

```bash
  pnpm i
```

2. Create `.env` config file in the root directory:

```ini
OSC_PORT=9000
OSC_HOST=127.0.0.1
HEART_RATE_DEVICE_NAME="YOUR_DEVICE_NAME"
```

> The `HEART_RATE_DEVICE_NAME` should match exactly with your Bluetooth device's name.

## Usage

Start the application:

```bash
  pnpm start
```

The application will:
1. Search for your heart rate device
2. Connect and start receiving BPM data
3. Send the heart rate to VRChat via OSC
4. Store readings in a local SQLite database

## Troubleshooting

- Make sure your heart rate device is powered on and in pairing mode
- Verify that VRChat is properly configured to receive OSC messages

## License

MIT
