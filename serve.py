import http.server
import socketserver

PORT = 8080
DIRECTORY = "dist" # This points to the folder Trunk builds into

class WASMHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        # Tell the server to look in the 'dist' folder
        super().__init__(*args, directory=DIRECTORY, **kwargs)

    def end_headers(self):
        # Add CORS headers so your phone doesn't block the request
        self.send_header('Access-Control-Allow-Origin', '*')
        super().end_headers()

# This is the "Magic" part that fixes the black screen
WASMHandler.extensions_map.update({
    ".wasm": "application/wasm",
    ".js": "application/javascript",
})

print(f"VEXT PROXY starting at http://0.0.0.0:{PORT}")
print(f"Point your phone to: http://192.168.1.42:{PORT}")

with socketserver.TCPServer(("0.0.0.0", PORT), WASMHandler) as httpd:
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down VEXT PROXY.")
        httpd.shutdown()