# HeartIO

Real-time heart rate monitoring that connects to Bluetooth devices and displays your heart rate in VRChat via OSC.

## Features

- Connect to BLE heart rate devices:
  - Heart rate monitors: Polar, CooSpo, Garmin, etc.
  - Smart wearables with BLE heart rate broadcast: Huawei bands, etc.
- Display heart rate in VRChat via OSC protocol
- Dynamic heart display styles based on BPM
- SQLite data logging for heart rate tracking
- Web-based graph visualization

## Quick Start (Recommended)

For users who prefer a simple setup without development environment, download the pre-built Rust binary from [GitHub Releases](https://github.com/xn-sakina/vrchat-heartio/releases).

### Rust Binary Version

The Rust version provides the same functionality as the Node.js CLI in a single executable file. When you run the Rust binary, it will automatically generate a `heartio.config.json` configuration file in the current directory. The heart rate threshold key in the config represents "less than" values.

```jsonc
  "HEART_RATE_LABEL": {
    // < 70
    "70": [],
    // 70 < bpm < 80
    "80": [],
  }
```

#### Xiaomi Band Support

Xiaomi bands have limited compatibility:
- **Xiaomi Band ≤7**: Supported via Bluetooth broadcast
- **Xiaomi Band ≥8**: Not supported (no Bluetooth broadcast capability)

For Xiaomi Band 6/7 users, detailed setup in the [Heartio Android App](./app-andriod/README.md).

## Node.js CLI Version

### Setup

Install dependencies:

```bash
  just install
```

### Configuration

Create `.env` file in the `cli` directory:

```ini
OSC_PORT=9000
OSC_HOST=0.0.0.0
# Configure ONE of the following:
HEART_RATE_DEVICE_NAME="YOUR_DEVICE_NAME"
# OR
# HEART_RATE_DEVICE_ADDRESS="YOUR_DEVICE_ADDRESS"
```

Configuration options:

- You can use either device name or address for connection
- If both name and address are omitted, the app will automatically connect to any available device with heart rate service (explicit configuration is recommended)

### Usage

Start the Node.js CLI:

```bash
  just start
```

## Graph Visualization

View heart rate data in a web interface:

```bash
  just start-graph
```

## Troubleshooting

- **OSC not working**: Ensure OSC is enabled in VRChat
- **Bluetooth not working**: Verify heart rate broadcasting is enabled on your wearable device
- **Device not found**: The Bluetooth reception range of most motherboards is very limited. If your device is not detected, try moving closer to your computer or consider purchasing a USB Bluetooth adapter for better range and reliability
- **Connection issues**: Make sure your heart rate device is not connected to other applications

## Development Notes

All code except the Node.js CLI was generated through vibe coding:
- **Apple Watch App**、**Graph Visualization**: Created with GPT-4o
- **Rust Version**: Claude Sonnet 4

## License

MIT
