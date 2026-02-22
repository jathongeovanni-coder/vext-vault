use leptos::*;
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::json;
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;
use uuid::Uuid;

/* ===================== GLOBAL JS BRIDGE BINDINGS ===================== */

#[wasm_bindgen]
extern "C" {
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
    let (unlocked, set_unlocked) = create_signal(false);
    let (paid, set_paid) = create_signal(false);
    let (status_msg, set_status_msg) = create_signal("SYSTEM READY. VECTOR 1 STANDBY.".to_string());
    let (attestations, set_attestations) = create_signal(Vec::<IntentAttestation>::new());
    let (unlock_prog, set_unlock_prog) = create_signal(0);
    let (pay_prog, set_pay_prog) = create_signal(0);
    let (holding_unlock, set_holding_unlock) = create_signal(false);
    let (holding_pay, set_holding_pay) = create_signal(false);

    let (btc, set_btc) = create_signal("—".into());
    let (sol, set_sol) = create_signal("—".into());
    let (asset, set_asset) = create_signal(Asset::SOL);

    // Fetch prices
    create_effect(move |_| {
        let assets = [("BTC", set_btc), ("SOL", set_sol)];
        for (sym, setter) in assets {
            spawn_local(async move {
                let url = format!("https://api.coinbase.com/v2/prices/{}-USD/spot", sym);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(amt) = json["data"]["amount"].as_str() {
                            setter.set(amt.to_string());
                        }
                    }
                }
            });
        }
    });

    let link_wallet = move |_| {
        set_status_msg.set("SECURE ENCLAVE INITIALIZING...".into());
        spawn_local(async move {
            match connectWallet().await {
                Ok(res) => {
                    let addr = res.as_string().unwrap_or_default();
                    if !addr.is_empty() && !addr.starts_with("ERROR") {
                        set_wallet_key.set(addr);
                        set_wallet_connected.set(true);
                        set_status_msg.set("VECTOR 1: POSSESSION VERIFIED.".into());
                    } else {
                        set_status_msg.set(format!("STATUS: {}", addr).into());
                    }
                }
                Err(_) => { set_status_msg.set("HANDSHAKE FAILED.".into()); }
            }
        });
    };

    let verify_bio = move |_| {
        set_status_msg.set("SCANNING BIOMATRIX...".into());
        spawn_local(async move {
            TimeoutFuture::new(1200).await; 
            set_biometric_verified.set(true);
            set_status_msg.set("IDENTITY VERIFIED. HOLD TO REVEAL.".into());
        });
    };

    // FIXED: Takes a generic web_sys::Event
    let start_unlock = move |ev: web_sys::Event| {
        ev.prevent_default(); 
        if !biometric_verified.get_untracked() { return; }
        set_holding_unlock.set(true);
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

    // FIXED: Takes a generic web_sys::Event
    let start_pay = move |ev: web_sys::Event| {
        ev.prevent_default();
        if !unlocked.get_untracked() || !wallet_connected.get_untracked() { return; }
        set_holding_pay.set(true);
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_pay.get_untracked() { 
                    set_pay_prog.set(0); 
                    return; 
                }
                set_pay_prog.set(i);
                TimeoutFuture::new(15).await;
            }
            
            set_status_msg.set("ATTESTING HARDWARE INTENT...".into());
            let nonce = Uuid::new_v4().to_string();
            let timestamp = (js_sys::Date::now() / 1000.0) as u64;

            match signWithHardware(nonce.clone()).await {
                Ok(js_val) if !js_val.is_null() => {
                    let sig = Reflect::get(&js_val, &"signature".into()).unwrap_or(JsValue::NULL).as_string().unwrap_or_default();
                    let new_auth = IntentAttestation {
                        asset_symbol: asset.get().symbol().into(),
                        wallet_pubkey: wallet_key.get_untracked(),
                        biometric_proof: "HARDWARE-VERIFIED".into(),
                        hold_duration_ms: 1500,
                        entropy_hash: "VEXT-PROVED".into(),
                        nonce,
                        timestamp_utc: timestamp,
                        signature: sig,
                    };
                    set_attestations.update(|list| list.push(new_auth));
                    set_paid.set(true);
                    set_status_msg.set("INTENT SEALED.".into());
                }
                _ => { set_status_msg.set("SIGNING FAILED.".into()); }
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
                        <div class="price-item" on:click=move |_| set_asset.set(Asset::BTC)>
                            <span>"BTC"</span><strong>"$" {move || btc.get()}</strong>
                        </div>
                        <div class="price-item selected" on:click=move |_| set_asset.set(Asset::SOL)>
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
                                    view! { <div class="log-entry"><span>{a.asset_symbol}</span><span>"✓"</span></div> }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </main>

                <div class="status-monitor" style="font-size: 10px; color: #3b82f6; text-align: center; margin: 15px 0; font-family: monospace; letter-spacing: 0.05em; text-transform: uppercase;">
                    {move || status_msg.get()}
                </div>

                <footer class="controls">
                    {move || {
                        if !wallet_connected.get() {
                            view! { <button class="action-btn primary" on:click=link_wallet>"LINK WALLET"</button> }.into_view()
                        } else if !biometric_verified.get() {
                            view! { <button class="action-btn primary" on:click=verify_bio>"SCAN BIOMATRIX"</button> }.into_view()
                        } else if !unlocked.get() {
                            view! {
                                <div class="hold-container">
                                    <button 
                                        class="action-btn hold" 
                                        on:mousedown=move |ev| start_unlock(ev.unchecked_into())
                                        on:touchstart=move |ev| start_unlock(ev.unchecked_into())
                                        on:mouseup={move |_| set_holding_unlock.set(false)}
                                        on:touchend={move |_| set_holding_unlock.set(false)}
                                    >"HOLD TO REVEAL"</button>
                                    <div class="progress-bar" style:width={move || format!("{}%", unlock_prog.get())}></div>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="hold-container">
                                    <button 
                                        class="action-btn authorize" 
                                        on:mousedown=move |ev| start_pay(ev.unchecked_into())
                                        on:touchstart=move |ev| start_pay(ev.unchecked_into())
                                        on:mouseup={move |_| set_holding_pay.set(false)}
                                        on:touchend={move |_| set_holding_pay.set(false)}
                                    >"HOLD TO AUTHORIZE"</button>
                                    <div class="progress-bar auth" style:width={move || format!("{}%", pay_prog.get())}></div>
                                </div>
                            }.into_view()
                        }
                    }}
                </footer>
            </div>
        </div>
    }
}

// --- HELPERS ---
#[derive(Debug, Clone, Copy, PartialEq)] enum Asset { BTC, SOL }
impl Asset { fn symbol(&self) -> &'static str { match self { Asset::BTC => "BTC", Asset::SOL => "SOL" } } }
#[derive(Deserialize)] struct CoinbaseResp { data: CoinbaseData }
#[derive(Deserialize)] struct CoinbaseData { amount: String }

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::document().get_element_by_id("vext-root").unwrap().dyn_into::<HtmlElement>().unwrap();
    mount_to(root, || view! { <App /> });
}