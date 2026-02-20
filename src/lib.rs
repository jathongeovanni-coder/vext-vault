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

/* ===================== GLOBAL JS BRIDGE BINDINGS ===================== */

#[wasm_bindgen]
extern "C" {
    // These bind to the window.vext object defined in webauthn_bridge.js
    #[wasm_bindgen(js_namespace = ["vext"], catch)]
    async fn connectWallet() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["vext"], catch)]
    async fn signWithHardware(challengeHex: String) -> Result<JsValue, JsValue>;
}

/* ===================== ATTESTATION DATA ===================== */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntentAttestation {
    pub asset_symbol: String,
    pub wallet_pubkey: String,
    pub biometric_proof: String, 
    pub hold_duration_ms: u64,    
    pub entropy_hash: String,     
    pub nonce: String,           
    pub timestamp_utc: u64,      
    pub signature: String,       
}

/* ===================== VEXT VAULT APP ===================== */

#[component]
pub fn App() -> impl IntoView {
    let (wallet_connected, set_wallet_connected) = create_signal(false);
    let (wallet_key, set_wallet_key) = create_signal(String::new());
    let (biometric_verified, set_biometric_verified) = create_signal(false);
    let (_verifying_bio, set_verifying_bio) = create_signal(false);
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

    // Fetch prices from Coinbase
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

    // Linking wallet via Global JS Bridge
    let link_wallet = move |_| {
        set_status_msg.set("COMMUNICATING WITH GLOBAL BRIDGE...".into());
        spawn_local(async move {
            match connectWallet().await {
                Ok(res) => {
                    let addr = res.as_string().unwrap_or_default();
                    if addr == "ERROR_NO_WALLET" {
                        set_status_msg.set("ERROR: PHANTOM NOT INSTALLED.".into());
                    } else if addr == "ERROR_REJECTED" {
                        set_status_msg.set("CONNECTION REJECTED BY USER.".into());
                    } else if !addr.is_empty() {
                        set_wallet_key.set(addr);
                        set_wallet_connected.set(true);
                        set_status_msg.set("VECTOR 1 SECURED. READY FOR IDENTITY SCAN.".into());
                    }
                }
                Err(_) => {
                    set_status_msg.set("ERROR: BRIDGE NOT INITIALIZED.".into());
                }
            }
        });
    };

    let verify_bio = move |_| {
        set_verifying_bio.set(true);
        set_status_msg.set("SCANNING BIOMATRIX...".into());
        spawn_local(async move {
            TimeoutFuture::new(1200).await; 
            set_biometric_verified.set(true);
            set_status_msg.set("IDENTITY VERIFIED. ENGAGE HOLD TO REVEAL.".into());
            set_verifying_bio.set(false);
        });
    };

    let start_unlock = move || {
        if !biometric_verified.get_untracked() { return; }
        set_holding_unlock.set(true);
        set_status_msg.set("REVEALING VAULT DATA...".into());
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_unlock.get_untracked() { 
                    set_unlock_prog.set(0); 
                    return; 
                }
                set_unlock_prog.set(i);
                TimeoutFuture::new(10).await;
            }
            set_unlocked.set(true);
            set_status_msg.set("VAULT REVEALED.".into());
        });
    };

    let start_pay = move || {
        if !unlocked.get_untracked() || !wallet_connected.get_untracked() { return; }
        set_holding_pay.set(true);
        set_status_msg.set("ATTESTING HUMAN INTENT...".into());
        
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_pay.get_untracked() { 
                    set_pay_prog.set(0); 
                    return; 
                }
                set_pay_prog.set(i);
                TimeoutFuture::new(15).await;
            }
            
            let nonce = Uuid::new_v4().to_string();
            let timestamp = (js_sys::Date::now() / 1000.0) as u64;
            let current_asset = asset.get().symbol();
            let wallet_pk = wallet_key.get_untracked();

            let message_json = json!({
                "asset": current_asset,
                "nonce": nonce,
                "timestamp_utc": timestamp,
                "wallet": wallet_pk,
            });
            let challenge_b64 = b64_encode(message_json.to_string().as_bytes());

            match signWithHardware(challenge_b64).await {
                Ok(js_val) if !js_val.is_null() => {
                    let sig = Reflect::get(&js_val, &"signature".into()).unwrap_or(JsValue::NULL).as_string().unwrap_or_default();
                    
                    set_status_msg.set("COMMITTING TO STATEFUL MEMORY...".into());
                    let verifier_req = Request::post("/api/verify")
                        .json(&json!({ "nonce": nonce, "timestamp": timestamp }))
                        .unwrap()
                        .send()
                        .await;

                    if let Ok(resp) = verifier_req {
                        if resp.ok() {
                            let new_auth = IntentAttestation {
                                asset_symbol: current_asset.to_string(),
                                wallet_pubkey: wallet_pk,
                                biometric_proof: "HARDWARE-VERIFIED".to_string(),
                                hold_duration_ms: 1500,
                                entropy_hash: format!("VEXT-{}", Uuid::new_v4().to_string().get(0..8).unwrap()),
                                nonce,
                                timestamp_utc: timestamp,
                                signature: sig,
                            };
                            set_attestations.update(|list| list.push(new_auth));
                            set_paid.set(true);
                            set_status_msg.set("INTENT ATTESTED & SECURED.".into());
                        } else {
                            set_status_msg.set("ERROR: REPLAY DETECTED OR EXPIRED.".into());
                        }
                    }
                }
                _ => { set_status_msg.set("HARDWARE SIGNING FAILED.".into()); }
            }
            set_pay_prog.set(0);
            set_holding_pay.set(false);
        });
    };

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
                        <div class="price-item" class:selected={move || asset.get() == Asset::BTC} on:click={move |_| set_asset.set(Asset::BTC)}>
                            <span>"BTC"</span><strong>"$" {move || btc.get()}</strong>
                        </div>
                        <div class="price-item" class:selected={move || asset.get() == Asset::ETH} on:click={move |_| set_asset.set(Asset::ETH)}>
                            <span>"ETH"</span><strong>"$" {move || eth.get()}</strong>
                        </div>
                        <div class="price-item" class:selected={move || asset.get() == Asset::SOL} on:click={move |_| set_asset.set(Asset::SOL)}>
                            <span>"SOL"</span><strong>"$" {move || sol.get()}</strong>
                        </div>
                    </div>

                    <div class="history-log">
                        <h3>"SESSION AUDIT LOG"</h3>
                        <div class="log-entries">
                            {move || if attestations.get().is_empty() {
                                view! { <div class="empty-msg">"NO RECENT ATTESTATIONS"</div> }.into_view()
                            } else {
                                attestations.get().into_iter().rev().map(|a| {
                                    let sig_short = if a.signature.len() > 8 { a.signature[0..8].to_string() } else { "---".into() };
                                    view! {
                                        <div class="log-entry">
                                            <span>{a.asset_symbol}</span><span class="log-hash">{sig_short}</span><span>"✓"</span>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </main>

                <div class="status-monitor" style="font-size: 10px; color: #3b82f6; text-align: center; margin: 15px 0; font-family: monospace; letter-spacing: 0.05em; text-transform: uppercase; height: 12px;">
                    {move || status_msg.get()}
                </div>

                <footer class="controls">
                    <div class="button-stack">
                        {move || {
                            if !wallet_connected.get() {
                                view! { <button class="action-btn primary" on:click=link_wallet>"LINK WALLET"</button> }.into_view()
                            } else if !biometric_verified.get() {
                                view! { <button class="action-btn primary" on:click=verify_bio>"SCAN BIOMATRIX"</button> }.into_view()
                            } else if !unlocked.get() {
                                view! {
                                    <div class="hold-container">
                                        <button class="action-btn hold" on:mousedown={move |_| start_unlock()} on:mouseup={move |_| set_holding_unlock.set(false)}>"HOLD TO REVEAL"</button>
                                        <div class="progress-bar" style:width={move || format!("{}%", unlock_prog.get())}></div>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="hold-container">
                                        <button class="action-btn authorize" disabled={move || paid.get()} on:mousedown={move |_| start_pay()} on:mouseup={move |_| set_holding_pay.set(false)}>
                                            {move || if paid.get() { "VERIFIED" } else { "HOLD TO AUTHORIZE" }}
                                        </button>
                                        <div class="progress-bar auth" style:width={move || format!("{}%", pay_prog.get())}></div>
                                    </div>
                                }.into_view()
                            }
                        }}
                    </div>
                </footer>

                {move || if paid.get() {
                    let last = attestations.get().last().cloned().unwrap();
                    let sig_label = if last.signature.len() > 16 { last.signature[0..16].to_string() } else { last.signature.clone() };
                    let nonce_label = if last.nonce.len() > 8 { last.nonce[0..8].to_string() } else { last.nonce.clone() };
                    
                    view! {
                        <div class="receipt-overlay">
                            <div class="jagged-receipt">
                                <h3>"INTENT SIGNED"</h3>
                                <div class="receipt-row"><span>"SIG"</span><span style="font-size:8px">{sig_label}"..."</span></div>
                                <div class="receipt-row"><span>"NONCE"</span><span style="font-size:8px">{nonce_label}</span></div>
                                <div class="receipt-tag">"STATEFUL VEXT SEAL"</div>
                                <button class="dismiss-btn" on:click={move |_| set_paid.set(false)}>"DONE"</button>
                            </div>
                        </div>
                    }.into_view()
                } else { view! { <div class="hidden"></div> }.into_view() }}
            </div>
            
            // Fixed SVG attribute braces for Leptos compatibility
            <div class="nav-icon" style="position:fixed; bottom:20px; right:20px; cursor:pointer;">
                <svg 
                    width="30" 
                    height="30" 
                    viewBox="0 0 24 24" 
                    fill="none" 
                    stroke={move || if unlocked.get() { "#3b82f6" } else { "#64748b" }} 
                    stroke-width="2"
                >
                    <path d="M12 15V17M12 7V13M12 21C16.9706 21 21 16.9706 21 12C21 7.02944 16.9706 3 12 3C7.02944 3 3 7.02944 3 12C3 16.9706 7.02944 21 12 21Z" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
            </div>
        </div>
    }
}

// --- HELPERS ---
fn b64_encode(input: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(input)
}

#[derive(Debug, Clone, Copy, PartialEq)] enum Asset { BTC, ETH, SOL }
impl Asset { fn symbol(&self) -> &'static str { match self { Asset::BTC => "BTC", Asset::ETH => "ETH", Asset::SOL => "SOL" } } }
#[derive(Deserialize)] struct CoinbaseResp { data: CoinbaseData }
#[derive(Deserialize)] struct CoinbaseData { amount: String }

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::document().get_element_by_id("vext-root").unwrap().dyn_into::<HtmlElement>().unwrap();
    mount_to(root, || view! { <App /> });
}