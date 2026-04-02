import jwt
import requests
import time

# This MUST match the JWT_SECRET in your Rust .env file
SECRET = "change-me-immediately"
PROXY_URL = "http://127.0.0.1:6969"

def create_token(role, sub="test_user"):
    payload = {
        "sub": sub,
        "role": role,
        "exp": int(time.time()) + 3600 # 1 hour expiration
    }
    return jwt.encode(payload, SECRET, algorithm="HS256")

print("🛡️  VANGUARD-GATE PENETRATION TEST 🛡️\n")

# --- TEST 1: The Hacker (No Token) ---
print("Test 1: Connecting without a token (Should be Blocked)")
res1 = requests.get(f"{PROXY_URL}/api")
print(f"Result -> Status: {res1.status_code} | Body: {res1.text}\n")

# --- TEST 2: Valid User accessing standard API ---
print("Test 2: Valid 'User' accessing /api (Should Route to 8080)")
user_token = create_token("user", "hrithik_standard")
headers = {"Authorization": f"Bearer {user_token}"}
res2 = requests.get(f"{PROXY_URL}/api", headers=headers)
print(f"Result -> Status: {res2.status_code} | Body: {res2.text}\n")

# --- TEST 3: Privilege Escalation (User attacking Vault) ---
print("Test 3: Valid 'User' attempting to access /vault (Should be Forbidden)")
res3 = requests.get(f"{PROXY_URL}/vault", headers=headers)
print(f"Result -> Status: {res3.status_code} | Body: {res3.text}\n")

# --- TEST 4: The Admin ---
print("Test 4: Valid 'Admin' accessing /vault (Should Route to 9090)")
admin_token = create_token("admin", "hrithik_admin")
admin_headers = {"Authorization": f"Bearer {admin_token}"}
res4 = requests.get(f"{PROXY_URL}/vault", headers=admin_headers)
print(f"Result -> Status: {res4.status_code} | Body: {res4.text}\n")