use leptos::*;
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Reflect, Uint8Array, Promise};
use serde::Deserialize;
use web_sys::HtmlElement;
use wasm_bindgen::JsCast;

/* ===================== WALLET BINDINGS ===================== */

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_namespace = window, js_name = solana)]
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

/* ===================== APP ===================== */

#[component]
pub fn App() -> impl IntoView {
    let (unlocked, set_unlocked) = create_signal(false);
    let (paid, set_paid) = create_signal(false);
    let (unlock_prog, set_unlock_prog) = create_signal(0);
    let (pay_prog, set_pay_prog) = create_signal(0);
    let (holding_unlock, set_holding_unlock) = create_signal(false);
    let (holding_pay, set_holding_pay) = create_signal(false);

    let (wallet_connected, set_wallet_connected) = create_signal(false);
    let (wallet_key, set_wallet_key) = create_signal(String::new());
    let (wallet_error, set_wallet_error) = create_signal(None::<String>);

    let (btc, set_btc) = create_signal("—".into());
    let (eth, set_eth) = create_signal("—".into());
    let (sol, set_sol) = create_signal("—".into());
    let (asset, set_asset) = create_signal(Asset::SOL);

    /* -------- Market Prices -------- */
    let fetch_all_prices = move || {
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

    create_effect(move |_| { fetch_all_prices(); });

    /* -------- Wallet Logic -------- */
    let connect_wallet = move |_| {
        spawn_local(async move {
            set_wallet_error.set(None);
            if SOLANA.with(|s| s.is_undefined()) {
                set_wallet_error.set(Some("No Solana wallet detected. Please install Phantom or Solflare.".into()));
                return;
            }
            let _ = try_connect_wallet(set_wallet_connected, set_wallet_key).await; 
        });
    };

    let sign_payment = move || {
        spawn_local(async move {
            let message = format!(
                "VEXT PAYMENT\nAsset: {}\nTimestamp: {}",
                asset.get().symbol(),
                js_sys::Date::now()
            );

            match try_sign_message(&message).await {
                Ok(_) => {
                    set_paid.set(true);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Signing error: {}", e).into());
                    set_wallet_error.set(Some(format!("Signing failed: {}", e)));
                }
            }
        });
    };

    /* -------- Hold Handlers -------- */
    let start_unlock = move || {
        set_holding_unlock.set(true);
        spawn_local(async move {
            for i in 1..=100 {
                if !holding_unlock.get_untracked() { 
                    set_unlock_prog.set(0); 
                    return; 
                }
                set_unlock_prog.set(i);
                TimeoutFuture::new(15).await;
            }
            set_unlocked.set(true);
        });
    };

    let start_pay = move || {
        if !wallet_connected.get_untracked() {
            set_wallet_error.set(Some("Please connect your wallet first".into()));
            return;
        }
        if !unlocked.get_untracked() { return; }
        
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
            sign_payment();
        });
    };

    view! {
        <div class="app">
            <h1>"VEXT VAULT"</h1>

            <button class="wallet" on:click=connect_wallet>
                {move || if wallet_connected.get() { 
                    let key = wallet_key.get();
                    if key.len() > 8 { format!("{}...{}", &key[..4], &key[key.len()-4..]) } else { "CONNECTED".into() }
                } else { "CONNECT WALLET".into() }}
            </button>

            {move || wallet_error.get().map(|err| view! { <div class="error">{err}</div> })}

            <div class="asset">
                <button on:click=move |_| set_asset.set(Asset::BTC) class:selected=move || asset.get() == Asset::BTC>
                    "BTC $" {btc}
                </button>
                <button on:click=move |_| set_asset.set(Asset::ETH) class:selected=move || asset.get() == Asset::ETH>
                    "ETH $" {eth}
                </button>
                <button on:click=move |_| set_asset.set(Asset::SOL) class:selected=move || asset.get() == Asset::SOL>
                    "SOL $" {sol}
                </button>
            </div>

            /* FIXED: Wrapped start_unlock() so it works for both Mouse and Touch */
            <button 
                class="unlock" 
                on:mousedown=move |_| start_unlock() 
                on:mouseup=move |_| set_holding_unlock.set(false)
                on:touchstart=move |_| start_unlock()
                on:touchend=move |_| set_holding_unlock.set(false)
            >
                {move || if unlocked.get() { "UNLOCKED ✓".into() } else { format!("HOLD TO UNLOCK {}%", unlock_prog.get()) }}
            </button>

            /* FIXED: Wrapped start_pay() so it works for both Mouse and Touch */
            <button 
                class="pay" 
                on:mousedown=move |_| start_pay() 
                on:mouseup=move |_| set_holding_pay.set(false)
                on:touchstart=move |_| start_pay()
                on:touchend=move |_| set_holding_pay.set(false)
                disabled=move || !unlocked.get()
            >
                {move || if paid.get() { "SIGNED ✓".into() } else { format!("HOLD TO SIGN & PAY {}%", pay_prog.get()) }}
            </button>
        </div>
    }
}

/* ===================== WALLET HELPERS ===================== */

async fn try_connect_wallet(
    set_connected: WriteSignal<bool>,
    set_key: WriteSignal<String>,
) -> Result<String, String> {
    SOLANA.with(|solana| {
        let connect_fn = Reflect::get(solana, &"connect".into())
            .map_err(|_| "Failed to get connect function")?;
        
        let promise = js_sys::Function::from(connect_fn)
            .call0(solana)
            .map_err(|_| "Failed to call connect")?;

        spawn_local(async move {
            let _ = match JsFuture::from(Promise::from(promise)).await {
                Ok(res) => {
                    let pk = Reflect::get(&res, &"publicKey".into()).unwrap();
                    let to_string = Reflect::get(&pk, &"toString".into()).unwrap();
                    let result = js_sys::Function::from(to_string).call0(&pk).unwrap();
                    let pk_str = result.as_string().unwrap_or_default();
                    set_key.set(pk_str.clone());
                    set_connected.set(true);
                    Ok(pk_str)
                }
                Err(_) => Err("User rejected connection".to_string())
            };
        });

        Err("Connecting...".to_string())
    })
}

async fn try_sign_message(message: &str) -> Result<JsValue, String> {
    SOLANA.with(|solana| {
        let bytes = Uint8Array::from(message.as_bytes());
        let sign_fn = Reflect::get(solana, &"signMessage".into())
            .map_err(|_| "Failed to get signMessage function")?;

        let promise = js_sys::Function::from(sign_fn)
            .call1(solana, &bytes.into())
            .map_err(|_| "Failed to call signMessage")?;

        spawn_local(async move {
            // FIXED: Added semicolon and underscore to satisfy return type ()
            let _ = JsFuture::from(Promise::from(promise)).await;
        });

        Err("Signing...".to_string())
    })
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    let root = leptos::document().get_element_by_id("vext-root").unwrap().dyn_into::<HtmlElement>().unwrap();
    mount_to(root, || view! { <App /> });
}