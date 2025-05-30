# HeartIO Apple Watch App

Experimental Apple Watch app for real-time heart rate monitoring.

## Important Limitations

⚠️ **Heart Rate Sampling Limitation**: Apple Watch can only capture heart rate data every 5-10 seconds during workout mode, which is the fastest available rate. This limitation means your heart rate won't be reflected in real-time in VRChat. If real-time heart rate is critical for your use case, consider using heart rate bands or other devices that provide second-level updates.

## Setup

### 1. Configure Network Address

Update the server address in `heartio/heartio Watch App/ContentView.swift`:

```swift
    guard let url = URL(string: "http://YOUR_PC_IP:2333/heart?bpm=\(bpm)") else { return }
    //                                  ^^^^^^^^^^
```

Replace `YOUR_PC_IP` with your PC's internal IP address. For stability, configure a fixed internal domain mapping through your router settings.

### 2. Build with Xcode

Build and deploy the app to your Apple Watch through Xcode.

### 3. Configure PC Environment

**For Rust version:**

Add to `heartio.config.json`:
```json
{
  "APPLE_WATCH": true
}
```

**For Node.js CLI:**

Add to `.env` file:
```
APPLE_WATCH=true
```

## How It Works

The Apple Watch app sends heart rate data to your PC via HTTP requests.
