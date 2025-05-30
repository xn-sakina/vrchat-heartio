# vrchat-heartio

Real-time heart rate monitoring that connects to Bluetooth devices and displays your heart rate in VRChat via OSC.

## Features

- Connect to BLE heart rate devices:
  - Heart rate monitors: Polar, CooSpo, Garmin, etc.
  - Smart wearables with BLE heart rate broadcast: Huawei bands, Xiaomi bands (≤ 7), etc.
- Display heart rate in VRChat via OSC protocol
- Dynamic heart display styles based on BPM
- SQLite data logging for tracking heart rate over time

## Setup

1. Install dependencies:

    ```bash
      pnpm i
    ```

2. Create `.env` config file:

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

## Usage

Start the application:

```bash
  pnpm start
```

## Troubleshooting

- Verify heart rate broadcasting is enabled on your wearable device
- OSC must be enabled in VRChat settings

## License

MIT
