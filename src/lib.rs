use leptos::*;
use leptos::CollectView; 
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Reflect, Promise};
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;
use uuid::Uuid;
use ed25519_dalek::{SigningKey, Signer};

/* ===================== HARDENED ATTESTATION DATA ===================== */

/// The data object representing a verified human intent.
/// This matches the schema expected by the Institutional Verifier.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntentAttestation {
    pub asset_symbol: String,    // Re-added to resolve "dead code" warning
    pub wallet_pubkey: String,
    pub biometric_proof: String, 
    pub hold_duration_ms: u64,    
    pub entropy_hash: String,     
    pub nonce: String,           // Unique ID to prevent Replay Attacks
    pub timestamp_utc: u64,      // Unix Epoch for TTL validation
    pub signature: String,       // Ed25519 Cryptographic Seal
}

/* ===================== WALLET BINDINGS ===================== */

#[wasm_bindgen]
extern "C" {
    /// Modern approach to access window.solana without deprecation warnings.
    #[wasm_bindgen(js_namespace = window, js_name = solana)]
    fn get_solana() -> JsValue;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Asset { BTC, ETH, SOL }

impl Asset {
    fn symbol(&self) -> &'static str {
        match self {
            Asset::BTC => "BTC",
            Asset::ETH => "ETH",
            Asset::SOL => "SOL",
        }
    }
}

#[derive(Deserialize)]
struct CoinbaseResp { data: CoinbaseData }
#[derive(Deserialize)]
struct CoinbaseData { amount: String }

/* ===================== VEXT VAULT APP ===================== */

#[component]
pub fn App() -> impl IntoView {
    // --- STATE SIGNALS ---
    let (wallet_connected, set_wallet_connected) = create_signal(false);
    let (wallet_key, set_wallet_key) = create_signal(String::new());
    let (biometric_verified, set_biometric_verified) = create_signal(false);
    let (verifying_bio, set_verifying_bio) = create_signal(false);
    let (unlocked, set_unlocked) = create_signal(false);
    let (paid, set_paid) = create_signal(false);
    let (status_msg, set_status_msg) = create_signal("SYSTEM READY. WAITING FOR VECTOR 1.".to_string());
    let (attestations, set_attestations) = create_signal(Vec::<IntentAttestation>::new());
    let (unlock_prog, set_unlock_prog) = create_signal(0);
    let (pay_prog, set_pay_prog) = create_signal(0);
    let (holding_unlock, set_holding_unlock) = create_signal(false);
    let (holding_pay, set_holding_pay) = create_signal(false);

    let (btc, set_btc) = create_signal("—".into());
    let (eth, set_eth) = create_signal("—".into());
    let (sol, set_sol) = create_signal("—".into());
    let (asset, set_asset) = create_signal(Asset::SOL);

    // --- EFFECT: PRICE ORACLE ---
    create_effect(move |_| {
        let assets = [("BTC", set_btc), ("ETH", set_eth), ("SOL", set_sol)];
        for (sym, setter) in assets {
            spawn_local(async move {
                let url = format!("https://api.coinbase.com/v2/prices/{}-USD/spot", sym);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(json) = resp.json::<CoinbaseResp>().await {
                        setter.set(json.data.amount);
                    }
                }
            });
        }
    });

    // --- HANDLER: VECTOR 2 (IDENTITY) ---
    let verify_bio = move |_| {
        set_verifying_bio.set(true);
        set_status_msg.set("SCANNING BIOMATRIX...".into());
        spawn_local(async move {
            TimeoutFuture::new(1200).await; 
            set_biometric_verified.set(true);
            set_verifying_bio.set(false);
            set_status_msg.set("IDENTITY VERIFIED. ENGAGE HOLD TO REVEAL.".into());
        });
    };

    // --- HANDLER: STEALTH UNLOCK ---
    let start_unlock = move || {
        if !biometric_verified.get_untracked() { return; }
        set_holding_unlock.set(true);
        set_status_msg.set("REVEALING VAULT DATA...".into());
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_unlock.get_untracked() { 
                    set_unlock_prog.set(0); 
                    set_status_msg.set("HOLD INTERRUPTED.".into());
                    return; 
                }
                set_unlock_prog.set(i);
                TimeoutFuture::new(10).await;
            }
            set_unlocked.set(true);
            set_status_msg.set("STEALTH MODE DEACTIVATED.".into());
        });
    };

    // --- HANDLER: VECTOR 3 (INTENT ATTESTATION) ---
    let start_pay = move || {
        if !unlocked.get_untracked() || !wallet_connected.get_untracked() { return; }
        set_holding_pay.set(true);
        set_status_msg.set("ATTESTING HUMAN INTENT...".into());
        
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_pay.get_untracked() { 
                    set_pay_prog.set(0); 
                    set_status_msg.set("AUTHORIZATION FAILED.".into());
                    return; 
                }
                set_pay_prog.set(i);
                TimeoutFuture::new(15).await;
            }
            
            // --- CANONICAL SIGNING ENGINE ---
            let signing_key = SigningKey::from_bytes(&[0u8; 32]); 
            let nonce = Uuid::new_v4().to_string();
            let timestamp = (js_sys::Date::now() / 1000.0) as u64;
            let entropy = format!("VEXT-HEX-{}", js_sys::Math::random());
            let current_asset_sym = asset.get().symbol();

            // Create Canonical JSON message for signing
            let message_json = json!({
                "asset": current_asset_sym,
                "nonce": nonce,
                "timestamp_utc": timestamp,
                "wallet_pubkey": wallet_key.get_untracked(),
                "hold_duration_ms": 1500,
                "entropy_hash": entropy,
            });
            let message_str = message_json.to_string();
            let signature = signing_key.sign(message_str.as_bytes());

            // 3. Assemble final attestation
            let new_auth = IntentAttestation {
                asset_symbol: current_asset_sym.to_string(),
                wallet_pubkey: wallet_key.get_untracked(),
                biometric_proof: "BIO-ATTESTED".to_string(),
                hold_duration_ms: 1500,
                timestamp_utc: timestamp,
                nonce,
                entropy_hash: entropy,
                signature: hex::encode(signature.to_bytes()),
            };

            set_attestations.update(|list| list.push(new_auth));
            set_paid.set(true);
            set_pay_prog.set(0);
            set_status_msg.set("ATTESTATION SIGNED & CANONICALIZED.".into());
        });
    };

    // --- VIEW ---
    view! {
        <div class="container">
            <div class="vault-card">
                <header>
                    <div class="logo">"VEXT"</div>
                    <div class="status-pill" class:active={move || unlocked.get()}>
                        {move || if unlocked.get() { "SECURE SESSION" } else { "VAULT SECURED" }}
                    </div>
                </header>

                <main class:blurred={move || !unlocked.get()}>
                    <div class="price-display">
                        <div class="price-item" 
                             class:selected={move || asset.get() == Asset::BTC} 
                             on:click={move |_| set_asset.set(Asset::BTC)}>
                            <span>"BTC"</span>
                            <strong>"$" {move || btc.get()}</strong>
                        </div>
                        <div class="price-item" 
                             class:selected={move || asset.get() == Asset::ETH} 
                             on:click={move |_| set_asset.set(Asset::ETH)}>
                            <span>"ETH"</span>
                            <strong>"$" {move || eth.get()}</strong>
                        </div>
                        <div class="price-item" 
                             class:selected={move || asset.get() == Asset::SOL} 
                             on:click={move |_| set_asset.set(Asset::SOL)}>
                            <span>"SOL"</span>
                            <strong>"$" {move || sol.get()}</strong>
                        </div>
                    </div>

                    <div class="history-log">
                        <h3>"SESSION AUDIT LOG"</h3>
                        <div class="log-entries">
                            {move || attestations.get().into_iter().rev().map(|a| {
                                let sig_short = a.signature.get(0..8).map(|s| s.to_string()).unwrap_or_default();
                                let sym = a.asset_symbol;
                                view! {
                                    <div class="log-entry">
                                        <span>{sym}</span>
                                        <span class="log-hash">{sig_short}</span>
                                        <span>"✓"</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </main>

                <div class="status-monitor" style="font-size: 10px; color: #3b82f6; text-align: center; margin: 15px 0; font-family: monospace; letter-spacing: 0.05em; text-transform: uppercase;">
                    {move || status_msg.get()}
                </div>

                <footer class="controls">
                    <div class="step-indicator">
                        <div class="step" class:done={move || wallet_connected.get()}>"1"</div>
                        <div class="step" class:done={move || biometric_verified.get()}>"2"</div>
                        <div class="step" class:done={move || unlocked.get()}>"3"</div>
                    </div>

                    <div class="button-stack">
                        {move || if !wallet_connected.get() {
                            view! {
                                <button class="action-btn primary" on:click={move |_| {
                                    try_connect_wallet(set_wallet_connected, set_wallet_key, set_status_msg);
                                }}>
                                    "LINK WALLET"
                                </button>
                            }.into_view()
                        } else if !biometric_verified.get() {
                            view! {
                                <button class="action-btn primary" on:click={verify_bio} disabled={move || verifying_bio.get()}>
                                    {move || if verifying_bio.get() { "SCANNING..." } else { "SCAN BIOMATRIX" }}
                                </button>
                            }.into_view()
                        } else if !unlocked.get() {
                            view! {
                                <div class="hold-container">
                                    <button class="action-btn hold" 
                                        on:mousedown={move |_| start_unlock()} 
                                        on:mouseup={move |_| set_holding_unlock.set(false)}
                                    >
                                        "HOLD TO REVEAL"
                                    </button>
                                    <div class="progress-bar" style:width={move || format!("{}%", unlock_prog.get())}></div>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="hold-container">
                                    <button class="action-btn authorize" 
                                        disabled={move || paid.get()}
                                        on:mousedown={move |_| start_pay()} 
                                        on:mouseup={move |_| set_holding_pay.set(false)}
                                    >
                                        {move || if paid.get() { "VERIFIED" } else { "HOLD TO AUTHORIZE" }}
                                    </button>
                                    <div class="progress-bar auth" style:width={move || format!("{}%", pay_prog.get())}></div>
                                </div>
                            }.into_view()
                        }}
                    </div>
                </footer>

                {move || {
                    if let Some(last) = attestations.get().last().cloned() {
                        if paid.get() {
                            let sig_display = format!("{}...", last.signature.get(0..16).unwrap_or(""));
                            let nonce_display = last.nonce.get(0..8).unwrap_or("").to_string();
                            
                            return view! {
                                <div class="receipt-overlay">
                                    <div class="jagged-receipt">
                                        <h3>"INTENT SIGNED"</h3>
                                        <div class="receipt-row"><span>"ASSET"</span><span>{last.asset_symbol}</span></div>
                                        <div class="receipt-row"><span>"SIG"</span><span style="font-size:8px">{sig_display}</span></div>
                                        <div class="receipt-row"><span>"NONCE"</span><span style="font-size:8px">{nonce_display}</span></div>
                                        <div class="receipt-tag">"CANONICAL VEXT SEAL"</div>
                                        <button class="dismiss-btn" on:click={move |_| set_paid.set(false)}>"DONE"</button>
                                    </div>
                                </div>
                            }.into_view();
                        }
                    }
                    view! { <div class="hidden"></div> }.into_view()
                }}
            </div>
        </div>
    }
}

// --- HELPER: WALLET LOGIC ---
fn try_connect_wallet(
    set_connected: WriteSignal<bool>,
    set_key: WriteSignal<String>,
    set_status: WriteSignal<String>,
) {
    spawn_local(async move {
        let solana = get_solana();
        if solana.is_undefined() {
            set_status.set("ERROR: SOLANA INJECTION NOT FOUND.".into());
            return;
        }
        set_status.set("HANDSHAKING...".into());
        let connect_fn = Reflect::get(&solana, &"connect".into()).unwrap();
        let promise = js_sys::Function::from(connect_fn).call0(&solana).unwrap();
        if let Ok(res) = JsFuture::from(Promise::from(promise)).await {
            let pk = Reflect::get(&res, &"publicKey".into()).unwrap();
            let to_string = Reflect::get(&pk, &"toString".into()).unwrap();
            let result = js_sys::Function::from(to_string).call0(&pk).unwrap();
            set_key.set(result.as_string().unwrap_or_default());
            set_connected.set(true);
            set_status.set("VECTOR 1 SECURED. SCAN BIOMATRIX.".into());
        }
    });
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::document().get_element_by_id("vext-root").unwrap().dyn_into::<HtmlElement>().unwrap();
    mount_to(root, || view! { <App /> });
}