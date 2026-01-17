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
    #[wasm_bindgen(js_namespace = window, js_name = solana)]
    static SOLANA: JsValue;
}

/* ===================== DATA ===================== */

#[derive(Debug, Clone, Copy, PartialEq)]
enum Asset {
    BTC,
    ETH,
    SOL,
}

#[derive(Deserialize)]
struct CoinbaseResp {
    data: CoinbaseData,
}

#[derive(Deserialize)]
struct CoinbaseData {
    amount: String,
}

/* ===================== APP ===================== */

#[component]
pub fn App() -> impl IntoView {
    // Core state
    let (unlocked, set_unlocked) = create_signal(false);
    let (paid, set_paid) = create_signal(false);

    // Hold progress
    let (unlock_prog, set_unlock_prog) = create_signal(0);
    let (pay_prog, set_pay_prog) = create_signal(0);
    let (holding_unlock, set_holding_unlock) = create_signal(false);
    let (holding_pay, set_holding_pay) = create_signal(false);

    // Wallet
    let (wallet_connected, set_wallet_connected) = create_signal(false);
    let (wallet_key, set_wallet_key) = create_signal(String::new());

    // Market
    let (btc, set_btc) = create_signal("—".into());
    let (eth, set_eth) = create_signal("—".into());
    let (sol, set_sol) = create_signal("—".into());
    let (asset, set_asset) = create_signal(Asset::SOL);

    /* -------- Market Prices -------- */

    let fetch_price = |symbol: &'static str, setter: WriteSignal<String>| {
        spawn_local(async move {
            let url = format!(
                "https://api.coinbase.com/v2/prices/{}-USD/spot",
                symbol
            );

            if let Ok(resp) = Request::get(&url).send().await {
                if let Ok(json) = resp.json::<CoinbaseResp>().await {
                    setter.set(json.data.amount);
                }
            }
        });
    };

    create_effect(move |_| {
        fetch_price("BTC", set_btc);
        fetch_price("ETH", set_eth);
        fetch_price("SOL", set_sol);
    });

    /* -------- Wallet -------- */

    let connect_wallet = move |_| {
        spawn_local(async move {
            if SOLANA.is_undefined() {
                return;
            }

            let connect_fn = Reflect::get(&SOLANA, &"connect".into()).unwrap();
            let promise = js_sys::Function::from(connect_fn)
                .call0(&SOLANA)
                .unwrap();

            let res = JsFuture::from(Promise::from(promise))
                .await
                .unwrap();

            let pk = Reflect::get(&res, &"publicKey".into()).unwrap();
            set_wallet_key.set(pk.as_string().unwrap_or_default());
            set_wallet_connected.set(true);
        });
    };

    let sign_payment = move || {
        spawn_local(async move {
            let message = format!(
                "VEXT PAYMENT\nAsset: {:?}\nTimestamp: {}",
                asset.get(),
                js_sys::Date::now()
            );

            let bytes = Uint8Array::from(message.as_bytes());
            let sign_fn = Reflect::get(&SOLANA, &"signMessage".into()).unwrap();

            let promise = js_sys::Function::from(sign_fn)
                .call1(&SOLANA, &bytes.into())
                .unwrap();

            let _ = JsFuture::from(Promise::from(promise))
                .await
                .unwrap();

            set_paid.set(true);
        });
    };

    /* -------- Hold Logic -------- */

    let start_unlock = move |_| {
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

    let start_pay = move |_| {
        if !wallet_connected.get_untracked() || !unlocked.get_untracked() {
            return;
        }

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

    /* -------- UI -------- */

    return view! {
        <div class="min-h-screen bg-slate-950 text-white p-10 font-mono">
            <h1 class="text-3xl font-black mb-6">"VEXT VAULT"</h1>

            <button on:click=connect_wallet class="mb-6 px-4 py-2 bg-purple-600 rounded">
                {move || if wallet_connected.get() {
                    format!("CONNECTED {}", &wallet_key.get()[..6])
                } else {
                    "CONNECT WALLET".into()
                }}
            </button>

            <div class="flex gap-4 mb-6">
                <button on:click=move |_| set_asset.set(Asset::BTC)>"BTC $" {btc}</button>
                <button on:click=move |_| set_asset.set(Asset::ETH)>"ETH $" {eth}</button>
                <button on:click=move |_| set_asset.set(Asset::SOL)>"SOL $" {sol}</button>
            </div>

            <button
                on:mousedown=start_unlock
                on:mouseup=move |_| set_holding_unlock.set(false)
                class="w-full py-4 bg-slate-800 rounded mb-4"
            >
                "HOLD TO UNLOCK " {unlock_prog} "%"
            </button>

            <button
                on:mousedown=start_pay
                on:mouseup=move |_| set_holding_pay.set(false)
                disabled=move || !unlocked.get()
                class="w-full py-4 bg-green-600 rounded disabled:opacity-30"
            >
                {move || if paid.get() { "SIGNED ✓" } else { "HOLD TO SIGN & PAY" }}
                " " {pay_prog} "%"
            </button>
        </div>
    };
}

/* ===================== ENTRY ===================== */

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let root = leptos::document()
        .get_element_by_id("vext-root")
        .expect("vext-root not found")
        .dyn_into::<HtmlElement>()
        .expect("vext-root not HtmlElement");

    mount_to(root, || view! { <App /> });

    if let Some(loader) = leptos::document().get_element_by_id("bridge-loader") {
        loader.remove();
    }
}
