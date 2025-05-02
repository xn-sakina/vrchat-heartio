# vrchat-heartio

Monitor your heart rate and display it in VRChat.

> [!IMPORTANT]
> You should use bluetooth heart rate devices like `Polar`, `CooSpo`, or `Garmin`, which can get data directly without a server.

### Usage

1. install deps:

```bash
  pnpm i
```

2. create · config file

```ini
OSC_PORT=9000
OSC_HOST=0.0.0.0
HEART_RATE_DEVICE_NAME="XXXXXX"
```

3. run

```bash
  pnpm start
```

# License

MIT
