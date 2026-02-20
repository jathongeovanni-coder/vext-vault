 /**
 * VEXT Global Bridge (v3.0 - Bulletproof Edition)
 * Attaching to window to ensure WASM visibility.
 */

window.vext = {
    connectWallet: async function() {
        console.log("VEXT: Direct Handshake Initiated...");
        try {
            if (!window.solana) {
                console.error("VEXT: Phantom not found");
                return "ERROR_NO_WALLET";
            }
            const resp = await window.solana.connect();
            console.log("VEXT: Connection Success:", resp.publicKey.toString());
            return resp.publicKey.toString();
        } catch (err) {
            console.error("VEXT: Connection Rejected:", err);
            return "ERROR_REJECTED";
        }
    },

    signWithHardware: async function(challengeHex) {
        console.log("VEXT: Biometric Hardware Request...");
        try {
            const binaryString = atob(challengeHex);
            const challengeBytes = new Uint8Array(binaryString.length);
            for (let i = 0; i < binaryString.length; i++) {
                challengeBytes[i] = binaryString.charCodeAt(i);
            }

            const credential = await navigator.credentials.get({
                publicKey: {
                    challenge: challengeBytes,
                    userVerification: "required",
                    timeout: 60000,
                    rpId: window.location.hostname === "localhost" ? "localhost" : window.location.hostname
                }
            });

            if (!credential) return null;

            const response = credential.response;
            const bufferToBase64 = (buf) => window.btoa(String.fromCharCode(...new Uint8Array(buf)));

            return {
                signature: bufferToBase64(response.signature),
                clientData: bufferToBase64(response.clientDataJSON),
                authenticatorData: bufferToBase64(response.authenticatorData)
            };
        } catch (err) {
            console.error("VEXT: Hardware Error:", err);
            return null;
        }
    }
};