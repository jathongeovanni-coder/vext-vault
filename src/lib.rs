/**
 * VEXT Institutional Verifier (Backend)
 * This runs as a Vercel Serverless Function (Node.js) to check the Upstash "Memory."
 */

export default async function handler(req, res) {
  // 1. Security Headers (CORS)
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  
  // Handle preflight requests
  if (req.method === 'OPTIONS') {
    return res.status(200).end();
  }

  // Ensure we only accept POST requests
  if (req.method !== 'POST') {
    return res.status(405).json({ error: "METHOD_NOT_ALLOWED" });
  }

  // 2. Data Extraction
  // Vercel automatically parses JSON bodies if the Content-Type is application/json
  const { nonce, timestamp } = req.body || {};
  
  if (!nonce || !timestamp) {
    return res.status(400).json({ error: "MISSING_PARAMETERS" });
  }

  const REDIS_URL = process.env.UPSTASH_REDIS_REST_URL;
  const REDIS_TOKEN = process.env.UPSTASH_REDIS_REST_TOKEN;

  if (!REDIS_URL || !REDIS_TOKEN) {
    console.error("VEXT Error: Redis environment variables are not set.");
    return res.status(500).json({ error: "CONFIGURATION_ERROR" });
  }

  try {
    // 3. Freshness Check (5-minute window)
    const now = Math.floor(Date.now() / 1000);
    const diff = Math.abs(now - parseInt(timestamp));
    
    if (diff > 300) {
      return res.status(403).json({ error: "ATTESTATION_EXPIRED" });
    }

    // 4. Atomic Nonce Check (The "Memory" Part)
    // We try to SET the nonce. If it already exists, 'NX' makes the command fail.
    // EX 300 sets a 5-minute expiration so the memory cleans itself up.
    const redisEndpoint = `${REDIS_URL}/set/${nonce}/1/NX/EX/300`;
    
    const redisReq = await fetch(redisEndpoint, {
      headers: { Authorization: `Bearer ${REDIS_TOKEN}` }
    });

    const result = await redisReq.json();

    // Upstash returns { result: "OK" } if the key was set successfully
    // It returns { result: null } if the key already existed (Replay Attack)
    if (result && result.result === "OK") {
      return res.status(200).json({ 
        status: "VERIFIED", 
        msg: "INTENT_STATE_SECURED",
        timestamp: now 
      });
    } else {
      return res.status(401).json({ error: "REPLAY_ATTACK_DETECTED" });
    }

  } catch (err) {
    console.error("VEXT Verifier Error:", err);
    return res.status(500).json({ error: "INTERNAL_VERIFIER_ERROR" });
  }
}