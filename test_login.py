import urllib.request
import json

url = 'http://localhost:3000/api/auth/login'
data = {
    "username": "admin",
    "password": "Admin@123",
    "captcha_token": "debug",
    "captcha_answer": "debug",
    "device_fp": "debug"
}
req = urllib.request.Request(url, data=json.dumps(data).encode('utf-8'), headers={'Content-Type': 'application/json'})

try:
    with urllib.request.urlopen(req) as response:
        print("Status:", response.status)
        print("Body:", response.read().decode('utf-8'))
except urllib.error.HTTPError as e:
    print("HTTP Error:", e.code)
    print("Error Body:", e.read().decode('utf-8'))
except Exception as e:
    print("Other Error:", e)
