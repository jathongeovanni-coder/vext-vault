/**
 * VEXT WebAuthn Bridge (v2.0 Hardened)
 * * This module bridges the Rust WASM environment with the browser's 
 * Credential Management API. It ensures that the "Intent Attestation" 
 * is signed inside the device's Secure Enclave (TPM).
 */

/**
 * Signs a challenge using the device's hardware biometrics.
 * @param {string} challengeHex - Base64 encoded JSON string from Rust.
 * @param {string} walletAddress - The user's public key for context.
 */
export async function signWithHardware(challengeHex, walletAddress) {
    try {
        console.log("VEXT: Initializing Hardware Handshake...");

        // 1. Convert the Base64 challenge from Rust into a binary Uint8Array
        const binaryString = atob(challengeHex);
        const challengeBytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            challengeBytes[i] = binaryString.charCodeAt(i);
        }

        // 2. Trigger the native Biometric / Security Key popup
        // This is where the user sees "Verify your identity"
        const credential = await navigator.credentials.get({
            publicKey: {
                challenge: challengeBytes,
                userVerification: "required",
                timeout: 60000, // 1 minute timeout
                allowCredentials: [], // In production, we'd filter for registered keys
                rpId: window.location.hostname === "localhost" ? "localhost" : window.location.hostname
            }
        });

        if (!credential) {
            throw new Error("User cancelled or hardware timed out.");
        }

        // 3. Extract the cryptographic results generated INSIDE the hardware
        // We convert these binary buffers back to Base64 to send them back to Rust
        const response = credential.response;

        return {
            signature: arrayBufferToBase64(response.signature),
            clientData: arrayBufferToBase64(response.clientDataJSON),
            authenticatorData: arrayBufferToBase64(response.authenticatorData)
        };

    } catch (err) {
        console.error("VEXT Hardware Bridge Error:", err);
        // Returning null tells our Rust code that the authorization failed
        return null; 
    }
}

/**
 * Utility: Converts ArrayBuffer to Base64 string for Rust compatibility.
 */
function arrayBufferToBase64(buffer) {
    let binary = '';
    const bytes = new Uint8Array(buffer);
    const len = bytes.byteLength;
    for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
}