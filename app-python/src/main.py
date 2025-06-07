import asyncio
import signal
from bleak import BleakScanner
import time
import requests

stop_scanning = asyncio.Event()
last_seen = {}


def send_to_osc(bpm: int):
    url = f"http://127.0.0.1:2333/heart?bpm={bpm}"
    if bpm > 0:
        try:
            response = requests.get(url)
            if response.status_code == 200:
                print(f"Sent BPM {bpm} to OSC server successfully.")
            else:
                print(
                    f"Failed to send BPM {bpm}. Server responded with status code {response.status_code}."
                )
        except requests.RequestException as e:
            print(f"Error sending BPM {bpm}: {e}")


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
                    print(f"[{device.address}] Heart rate: {heart_rate} bpm")
                    send_to_osc(heart_rate)
                else:
                    print(f"[{device.address}] Manufacturer data too short: {value}")
        else:
            print(f"[{device.address}] No manufacturer data in advertisement")


async def main():
    print("Listening for Xiaomi Smart Band advertisements...")
    scanner = BleakScanner(detection_callback=handle_advertisement)
    await scanner.start()
    print("Scanner started.")
    try:
        await stop_scanning.wait()
    finally:
        print("Stopping scanner...")
        await scanner.stop()
        print("Scanner stopped.")


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
    finally:
        loop.close()
        print("Program exited.")
