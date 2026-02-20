/**
 * VEXT Global Bridge (v3.7 - Hardened Handshake)
 * This script handles the low-level handshakes between the Rust WASM engine,
 * the Phantom Wallet, and the Browser's Hardware Security Enclave (WebAuthn).
 */

(function() {
    console.log("🛡️ VEXT: Hardened Bridge Initializing...");

    window.vext = {
        /**
         * connectWallet: Initiates the Vector 1 Handshake.
         * Optimized with timeout handling and clean Promise execution 
         * to prevent WASM 'undefined' panics and popup suppression.
         */
        connectWallet: function() {
            console.log("VEXT: Wallet Handshake Request Received.");
            
            return new Promise((resolve) => {
                // Set a 30-second timeout in case the wallet extension hangs
                const timeout = setTimeout(() => {
                    console.error("VEXT: Wallet connection timeout");
                    resolve("ERROR_TIMEOUT");
                }, 30000);

                // 1. Check for Phantom/Solana injection
                if (!window.solana || !window.solana.isPhantom) {
                    console.error("VEXT: Phantom wallet extension not found.");
                    clearTimeout(timeout);
                    resolve("ERROR_NO_WALLET");
                    return; 
                }

                console.log("Phantom detected?", window.solana?.isPhantom);

                // 2. Request connection from the provider
                // Using .then/.catch instead of async executor for better timing stability
                window.solana.connect()
                    .then((resp) => {
                        clearTimeout(timeout);
                        const pubKey = resp.publicKey.toString();
                        console.log("VEXT: Wallet Handshake Success:", pubKey);
                        resolve(pubKey);
                    })
                    .catch((err) => {
                        clearTimeout(timeout);
                        console.error("VEXT: Wallet Handshake Error:", err);
                        
                        // Handle specific Phantom error codes (4001 = User Rejected)
                        if (err.code === 4001) {
                            resolve("ERROR_USER_REJECTED");
                        } else {
                            resolve("ERROR_REJECTED");
                        }
                    });
            });
        },

        /**
         * signWithHardware: Initiates the Vector 3 Hardware Attestation.
         * Binds the user's biometric scan to the cryptographic challenge.
         */
        signWithHardware: function(challengeHex) {
            console.log("VEXT: Hardware Attestation Initiated...");
            
            return new Promise((resolve) => {
                try {
                    // Convert the Base64 challenge from Rust into a binary Uint8Array
                    const binaryString = atob(challengeHex);
                    const challengeBytes = new Uint8Array(binaryString.length);
                    for (let i = 0; i < binaryString.length; i++) {
                        challengeBytes[i] = binaryString.charCodeAt(i);
                    }

                    // Request the biometric signature from the device's Secure Enclave
                    navigator.credentials.get({
                        publicKey: {
                            challenge: challengeBytes,
                            userVerification: "required",
                            timeout: 60000,
                            rpId: window.location.hostname === "localhost" ? "localhost" : window.location.hostname
                        }
                    }).then((credential) => {
                        if (!credential) {
                            console.warn("VEXT: Hardware Attestation cancelled by user.");
                            resolve(null);
                            return;
                        }

                        const response = credential.response;
                        const bufferToBase64 = (buf) => window.btoa(String.fromCharCode(...new Uint8Array(buf)));
                        
                        console.log("VEXT: Hardware Attestation Sealed.");
                        
                        resolve({
                            signature: bufferToBase64(response.signature),
                            clientData: bufferToBase64(response.clientDataJSON),
                            authenticatorData: bufferToBase64(response.authenticatorData)
                        });
                    }).catch((err) => {
                        console.error("VEXT: Hardware Attestation Failed:", err);
                        resolve(null);
                    });
                } catch (err) {
                    console.error("VEXT: Hardware Conversion Error:", err);
                    resolve(null);
                }
            });
        }
    };

    console.log("🛡️ VEXT: Bridge Fully Operational at Global Scope.");
})();