import asyncio
import signal
from bleak import BleakScanner
from bleak.exc import BleakError
import time
import requests
import sys

stop_scanning = asyncio.Event()
last_seen = {}


def send_to_osc(bpm: int):
    url = f"http://127.0.0.1:2333/heart?bpm={bpm}"
    if bpm > 0:
        try:
            response = requests.get(url)
            if response.status_code == 200:
                print(f"Successfully sent BPM {bpm} to HeartIO main program.")
            else:
                print(
                    f"Failed to send BPM {bpm}. Server responded with status code {response.status_code}."
                )
        except requests.RequestException as e:
            print(f"Error sending BPM {bpm} to HeartIO: {e}")


def handle_advertisement(device, advertisement_data):
    now = time.time()
    addr = device.address
    if now - last_seen.get(addr, 0) < 1:
        return
    last_seen[addr] = now
    name = getattr(device, "name", "") or ""
    if "Xiaomi Smart Band" in name:
        mdata = advertisement_data.manufacturer_data
        if mdata:
            for _, value in mdata.items():
                if len(value) >= 4:
                    heart_rate = value[3]
                    print(f"[{device.address}] Received heart rate: {heart_rate} bpm")
                    send_to_osc(heart_rate)
                else:
                    print(f"[{device.address}] Manufacturer data too short: {value}")
        else:
            print(f"[{device.address}] No manufacturer data in advertisement")


async def check_bluetooth_availability():
    """Check if Bluetooth is available and enabled"""
    try:
        # Try to create a scanner to test Bluetooth availability
        test_scanner = BleakScanner()
        await test_scanner.start()
        await test_scanner.stop()
        return True
    except BleakError as e:
        print(f"Bluetooth error: {e}")
        return False
    except Exception as e:
        print(f"Error checking Bluetooth availability: {e}")
        return False


async def main():
    print("=" * 60)
    print("HeartIO Python Reporter - Xiaomi Band Broadcaster")
    print("=" * 60)
    print("This is a reporter program that forwards Xiaomi band heart rate")
    print("broadcast data to the HeartIO main program.")
    print("")
    print("IMPORTANT: You must enable APPLE_WATCH: true in heartio.config.json")
    print("This program is specifically designed for Xiaomi bands only.")
    print("=" * 60)
    print("")
    
    print("Checking Bluetooth availability...")
    
    # Check if Bluetooth is available before proceeding
    if not await check_bluetooth_availability():
        print("Error: Bluetooth is not available or disabled. Please enable Bluetooth and try again.")
        sys.exit(1)
    
    print("Listening for Xiaomi Smart Band advertisements...")
    
    try:
        scanner = BleakScanner(detection_callback=handle_advertisement)
        await scanner.start()
        print("Scanner started. Waiting for Xiaomi band broadcasts...")
        try:
            await stop_scanning.wait()
        finally:
            print("Stopping scanner...")
            await scanner.stop()
            print("Scanner stopped.")
    except BleakError as e:
        print(f"Bluetooth scanner error: {e}")
        print("Please check if Bluetooth is enabled and try again.")
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {e}")
        sys.exit(1)


def shutdown():
    print("Shutdown signal received, preparing to stop...")
    stop_scanning.set()


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            loop.add_signal_handler(sig, shutdown)
        except NotImplementedError:
            signal.signal(sig, lambda s, f: shutdown())

    try:
        loop.run_until_complete(main())
    except KeyboardInterrupt:
        print("Program interrupted by user.")
    except Exception as e:
        print(f"Program failed with error: {e}")
        sys.exit(1)
    finally:
        loop.close()
        print("Program exited.")
