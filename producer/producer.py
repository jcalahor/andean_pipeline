from kafka import KafkaProducer
import json
import binascii
import datetime
import threading 
import time
import random

input_data = [
    {"Symbol": "IBM", "Price": 102.3},
    {"Symbol": "MSFT", "Price": 10.3},
    {"Symbol": "GOOG", "Price": 1002.3},
    {"Symbol": "META", "Price": 299.3},
    {"Symbol": "XYZ", "Price": 211.3},
    {"Symbol": "NVDA", "Price": 112.3},
    {"Symbol": "APPL", "Price": 233.3},
    {"Symbol": "LKM", "Price": 11.3},
    {"Symbol": "UTC", "Price": 33.3},
]


def run():
    LIMIT = 5000
    producer = KafkaProducer(bootstrap_servers='kafka:9092')
    
    for i in range(LIMIT):
        ts = int(datetime.datetime.now().strftime('%s'))
        for entry in input_data:
            entry["TimeStamp"] = ts + random.randint(1000, 10000000)
            output = json.dumps(entry).encode('utf-8')
            producer.send('input_readings', output)
        if i % 2000 == 0:
            print(i)

if __name__ == "__main__":
   run()