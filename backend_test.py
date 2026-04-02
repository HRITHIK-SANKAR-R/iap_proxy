# backend.py
from http.server import BaseHTTPRequestHandler, HTTPServer
import json

class SimpleAPI(BaseHTTPRequestHandler):
    def do_GET(self):
        # 1. Send the 200 OK status
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()

        # 2. Send the JSON payload
        response = {
            "status": "success",
            "message": "🔥 VanguardGate successfully verified your token and routed you here! 🔥",
            "server": "Internal API (Port 8080)"
        }
        self.wfile.write(json.dumps(response).encode('utf-8'))

print("🟢 Protected Backend API listening on 127.0.0.1:8080...")
HTTPServer(('127.0.0.1', 8080), SimpleAPI).serve_forever()