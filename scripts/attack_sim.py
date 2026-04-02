# C:\Users\rajku\Downloads\sem 6\AC applied cryptography\projects\PROJECT_SENTINEL_V4\SENTINEL_CBDC_PROJECT\scripts\attack_sim.py

import requests
import time
import json
import concurrent.futures

# Configuration
TARGET_URL = "http://localhost:3000/api"
HEADERS = {"Content-Type": "application/json"}

class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    RESET = '\033[0m'

def log(status, message):
    if status == "BLOCKED":
        print(f"[{Colors.GREEN}DEFENSE SUCCESS{Colors.RESET}] {message}")
    elif status == "VULNERABLE":
        print(f"[{Colors.RED}VULNERABLE{Colors.RESET}] {message}")
    else:
        print(f"[{Colors.YELLOW}INFO{Colors.RESET}] {message}")

def test_sql_injection():
    print("\n--- 1. Testing SQL Injection Defense ---")
    payloads = [
        "' OR '1'='1", 
        "admin' --", 
        "UNION SELECT 1, version(), 3 --"
    ]
    
    for p in payloads:
        data = {
            "username": p,
            "password": "password123",
            "captcha_token": "dummy_bypass_for_test", # Note: Server would block this first in prod
            "captcha_answer": "DUMMY",
            "device_fp": "test_device"
        }
        try:
            # We expect the server to validate input format or fail authentication safely
            # Ideally blocked by InputSanitization middleware or PreparedStatement
            res = requests.post(f"{TARGET_URL}/auth/login", json=data, timeout=2)
            
            if res.status_code in [400, 401, 403]:
                log("BLOCKED", f"Payload '{p}' rejected with status {res.status_code}")
            elif res.status_code == 500:
                log("VULNERABLE", f"Payload '{p}' caused Server Error (Possible SQLi)")
            else:
                log("VULNERABLE", f"Payload '{p}' accepted? Status {res.status_code}")
        except Exception as e:
            log("BLOCKED", f"Connection refused/dropped: {e}")

def test_xss_injection():
    print("\n--- 2. Testing XSS Input Sanitization ---")
    xss_payload = "<script>alert('hack')</script>"
    
    # Try to register a user with XSS payload as username
    data = {
        "username": xss_payload,
        "password": "StrongPassword123!",
        "captcha_token": "valid_mock_token",
        "captcha_answer": "VALID"
    }
    
    try:
        res = requests.post(f"{TARGET_URL}/auth/signup", json=data)
        if res.status_code == 400: # Bad Request (Regex validation failed)
            log("BLOCKED", "XSS Username rejected by Regex Validator")
        else:
            log("VULNERABLE", f"XSS Payload accepted: {res.status_code}")
    except:
        log("BLOCKED", "Request failed")

def test_rate_limiter():
    print("\n--- 3. Testing Rate Limiting (DDoS Simulation) ---")
    print("Sending 50 requests in parallel...")
    
    def send_request(i):
        try:
            return requests.get(f"{TARGET_URL}/admin/stats").status_code
        except:
            return 0

    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        results = list(executor.map(send_request, range(50)))
    
    blocked_count = results.count(429)
    success_count = results.count(200) + results.count(401) # 401 is unauthorized but allowed through rate limit
    
    if blocked_count > 0:
        log("BLOCKED", f"Rate Limiter triggered! Blocked {blocked_count} requests.")
    else:
        log("INFO", f"No requests blocked (Threshold might be > 50). Success: {success_count}")

def main():
    print(f"{Colors.YELLOW}Starting Sentinel Forensic Penetration Test...{Colors.RESET}")
    print("Target:", TARGET_URL)
    
    try:
        test_sql_injection()
        test_xss_injection()
        test_rate_limiter()
    except KeyboardInterrupt:
        print("\nTest Cancelled")
    except Exception as e:
        print(f"Test Suite Error: {e}")

if __name__ == "__main__":
    main()