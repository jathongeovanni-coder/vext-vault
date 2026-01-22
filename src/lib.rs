use leptos::*;
use leptos::CollectView; 
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Reflect, Promise};
use serde::{Deserialize, Serialize};
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;

/* ===================== TRIPLE-LOCK ATTESTATION DATA ===================== */

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntentAttestation {
    pub asset: String,
    pub price_at_auth: String,
    pub biometric_verified: bool, 
    pub wallet_linked: bool,      
    pub hold_duration_ms: u64,    
    pub timestamp: f64,
    pub entropy_hash: String,     
}

/* ===================== WALLET BINDINGS ===================== */

#[wasm_bindgen]
extern "C" {
    // Stable binding for injected window.solana (Phantom/Solflare)
    #[wasm_bindgen(js_namespace = window, js_name = solana)]
    static SOLANA: JsValue;
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
    // --- Vector 1: Possession (Wallet) ---
    let (wallet_connected, set_wallet_connected) = create_signal(false);
    let (wallet_key, set_wallet_key) = create_signal(String::new());
    
    // --- Vector 2: Identity (Biomatrix) ---
    let (biometric_verified, set_biometric_verified) = create_signal(false);
    let (verifying_bio, set_verifying_bio) = create_signal(false);

    // --- Vector 3: Intent (The VEXT Hold) ---
    let (unlocked, set_unlocked) = create_signal(false);
    let (paid, set_paid) = create_signal(false);
    
    // --- Session History (The Audit Log) ---
    let (attestations, set_attestations) = create_signal(Vec::<IntentAttestation>::new());
    
    // --- Progress & Interaction ---
    let (unlock_prog, set_unlock_prog) = create_signal(0);
    let (pay_prog, set_pay_prog) = create_signal(0);
    let (holding_unlock, set_holding_unlock) = create_signal(false);
    let (holding_pay, set_holding_pay) = create_signal(false);

    // --- Market Data ---
    let (btc, set_btc) = create_signal("—".into());
    let (eth, set_eth) = create_signal("—".into());
    let (sol, set_sol) = create_signal("—".into());
    let (asset, set_asset) = create_signal(Asset::SOL);

    /* -------- Market Feed Logic -------- */
    let fetch_prices = move || {
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
    };

    create_effect(move |_| { fetch_prices(); });

    /* -------- Security Step Handlers -------- */
    
    let verify_bio = move |_| {
        set_verifying_bio.set(true);
        spawn_local(async move {
            TimeoutFuture::new(1200).await; // Simulated High-Security Scan
            set_biometric_verified.set(true);
            set_verifying_bio.set(false);
        });
    };

    let start_unlock = move || {
        if !biometric_verified.get_untracked() { return; }
        set_holding_unlock.set(true);
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_unlock.get_untracked() { set_unlock_prog.set(0); return; }
                set_unlock_prog.set(i);
                TimeoutFuture::new(10).await;
            }
            set_unlocked.set(true);
        });
    };

    let start_pay = move || {
        if !unlocked.get_untracked() || !wallet_connected.get_untracked() { return; }
        set_holding_pay.set(true);
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_pay.get_untracked() { set_pay_prog.set(0); return; }
                set_pay_prog.set(i);
                TimeoutFuture::new(15).await;
            }
            
            // Create the Attestation (Patent Logic)
            let new_auth = IntentAttestation {
                asset: asset.get().symbol().to_string(),
                price_at_auth: match asset.get() {
                    Asset::BTC => btc.get(),
                    Asset::ETH => eth.get(),
                    Asset::SOL => sol.get(),
                },
                biometric_verified: true,
                wallet_linked: true,
                hold_duration_ms: 1500,
                timestamp: js_sys::Date::now(),
                entropy_hash: format!("VEXT-SEC-{}", js_sys::Math::random()),
            };

            set_attestations.update(|list| list.push(new_auth));
            set_paid.set(true);
            set_pay_prog.set(0);
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

                /* Main Display: Uses the .blurred class from style.css for Stealth Mode */
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

                    /* Session Audit Log: Permanent proof for the session */
                    <div class="history-log">
                        <h3>"SESSION AUDIT LOG"</h3>
                        <div class="log-entries">
                            {move || attestations.get().into_iter().rev().map(|a| {
                                view! {
                                    <div class="log-entry">
                                        <span>{a.asset}</span>
                                        <span class="log-hash">{a.entropy_hash}</span>
                                        <span>"✓"</span>
                                    </div>
                                }
                            }).collect_view()}
                            {move || if attestations.get().is_empty() { 
                                view! { <p class="empty-msg">"Waiting for authorization..."</p> }.into_view() 
                            } else { 
                                view! { <div></div> }.into_view() 
                            }}
                        </div>
                    </div>
                </main>

                <footer class="controls">
                    /* The 1-2-3 Step System */
                    <div class="step-indicator">
                        <div class="step" class:done={move || wallet_connected.get()}>"1"</div>
                        <div class="step" class:done={move || biometric_verified.get()}>"2"</div>
                        <div class="step" class:done={move || unlocked.get()}>"3"</div>
                    </div>

                    <div class="button-stack">
                        {move || if !wallet_connected.get() {
                            view! {
                                <button class="action-btn primary" on:click={move |_| {
                                    try_connect_wallet(set_wallet_connected, set_wallet_key);
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
                                        on:touchstart={move |_| start_unlock()} 
                                        on:touchend={move |_| set_holding_unlock.set(false)}
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
                                        on:touchstart={move |_| start_pay()} 
                                        on:touchend={move |_| set_holding_pay.set(false)}
                                    >
                                        {move || if paid.get() { "VERIFIED" } else { "HOLD TO AUTHORIZE" }}
                                    </button>
                                    <div class="progress-bar auth" style:width={move || format!("{}%", pay_prog.get())}></div>
                                </div>
                            }.into_view()
                        }}
                    </div>
                </footer>

                /* The Jagged Receipt: Safe Option rendering to prevent panics */
                {move || {
                    if let Some(last) = attestations.get().last().cloned() {
                        if paid.get() {
                            return view! {
                                <div class="receipt-overlay">
                                    <div class="jagged-receipt">
                                        <h3>"INTENT ATTESTED"</h3>
                                        <div class="receipt-row"><span>"Asset"</span><span>{last.asset}</span></div>
                                        <div class="receipt-row"><span>"Security"</span><span>"BIO-VERIFIED"</span></div>
                                        <div class="receipt-row"><span>"Identity"</span><span>"VALIDATED"</span></div>
                                        <div class="receipt-tag">"VEXT SECURE BRIDGE"</div>
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

/* ===================== WALLET HELPERS ===================== */

fn try_connect_wallet(
    set_connected: WriteSignal<bool>,
    set_key: WriteSignal<String>,
) {
    spawn_local(async move {
        if SOLANA.is_undefined() {
            web_sys::console::error_1(&"VEXT System Error: window.solana not found.".into());
            return;
        }

        let connect_fn = Reflect::get(&SOLANA, &"connect".into()).unwrap();
        let promise = js_sys::Function::from(connect_fn)
            .call0(&SOLANA)
            .unwrap();

        if let Ok(res) = JsFuture::from(Promise::from(promise)).await {
            let pk = Reflect::get(&res, &"publicKey".into()).unwrap();
            let to_string = Reflect::get(&pk, &"toString".into()).unwrap();
            let result = js_sys::Function::from(to_string).call0(&pk).unwrap();

            set_key.set(result.as_string().unwrap_or_default());
            set_connected.set(true);
        }
    });
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::document().get_element_by_id("vext-root").unwrap().dyn_into::<HtmlElement>().unwrap();
    mount_to(root, || view! { <App /> });
}