import sqlite3
import json
from datetime import datetime, timedelta
import os

DB_PATH = os.path.join(os.path.dirname(__file__), '../../cache/data.sqlite') 
OUTPUT_FILE = os.path.join(os.path.dirname(__file__), '../src/heart_rate_data.json')
HOURS = 5

since_time = datetime.utcnow() - timedelta(hours=HOURS)
since_time_str = since_time.strftime('%Y-%m-%d %H:%M:%S')

conn = sqlite3.connect(DB_PATH)
cursor = conn.cursor()

cursor.execute('''
    SELECT bpm, strftime('%s', created_at) AS timestamp
    FROM heart_rate
    WHERE datetime(created_at) >= datetime(?)
    ORDER BY created_at ASC
''', (since_time_str,))

rows = cursor.fetchall()

results = [{'time': int(timestamp), 'bpm': bpm} for bpm, timestamp in rows]

with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
    json.dump(results, f, ensure_ascii=False, indent=2)

cursor.close()
conn.close()
