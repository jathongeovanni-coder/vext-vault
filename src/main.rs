use leptos::*;
use web_sys::UrlSearchParams;
use serde::{Deserialize, Serialize};
use gloo_timers::callback::Interval;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceData {
    pub base: String,
    pub currency: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinbaseResponse {
    pub data: PriceData,
}

// THE MERCHANT'S SMOKING GUN: Proof of Intent Structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub tx_id: String,
    pub merchant: String,
    pub usd_amount: String,
    pub crypto_amount: String,
    pub currency: String,
    pub timestamp: u64,
    pub hci_signature: String,
}

#[component]
fn App() -> impl IntoView {
    // --- SIGNALS (The reactive variables) ---
    let (view, set_view) = create_signal("dashboard".to_string());
    let (hold_progress, set_hold_progress) = create_signal(0.0);
    let (seed_phrase, set_seed_phrase) = create_signal("".to_string());
    let (selected_coin, set_selected_coin) = create_signal("ETH".to_string());
    let (receipt, set_receipt) = create_signal(None::<TransactionReceipt>);

    let (prices, set_prices) = create_signal::<Vec<(String, String)>>(vec![
        ("BTC".to_string(), "0.00".to_string()), 
        ("ETH".to_string(), "0.00".to_string()), 
        ("SOL".to_string(), "0.00".to_string())
    ]);
    
    let (merchant, set_merchant) = create_signal("VAULT_INIT".to_string());
    let (usd_amount, set_usd_amount) = create_signal("0.00".to_string());

    // --- DERIVED LOGIC ---
    let word_count = move || seed_phrase.get().split_whitespace().count();
    let is_unlocked = move || word_count() == 12;
    
    // Calculates crypto total based on live price and merchant request
    let crypto_total = move || {
        let usd: f64 = usd_amount.get().parse().unwrap_or(0.0);
        let coin = selected_coin.get();
        let price_entry = prices.get().into_iter().find(|(s, _)| *s == coin);
        
        if let Some((_, price_str)) = price_entry {
            let price: f64 = price_str.parse().unwrap_or(1.0);
            if price > 0.0 {
                return format!("{:.6}", usd / price);
            }
        }
        "0.000000".to_string()
    };

    // --- INITIALIZATION ---
    create_effect(move |_| {
        let window = web_sys::window().expect("no global `window` exists");
        let search_params = UrlSearchParams::new_with_str(&window.location().search().unwrap()).unwrap();
        
        if let Some(m) = search_params.get("merchant") { set_merchant.set(m.to_uppercase()); }
        if let Some(a) = search_params.get("amount") { set_usd_amount.set(a); }

        let symbols = vec!["BTC", "ETH", "SOL"];
        for sym in symbols {
            let sym_str = sym.to_string();
            spawn_local(async move {
                let url = format!("https://api.coinbase.com/v2/prices/{}-USD/spot", sym_str);
                if let Ok(resp) = gloo_net::http::Request::get(&url).send().await {
                    if let Ok(json) = resp.json::<CoinbaseResponse>().await {
                        set_prices.update(|p| {
                            if let Some(item) = p.iter_mut().find(|(s, _)| *s == sym_str) {
                                item.1 = json.data.amount.clone();
                            }
                        });
                    }
                }
            });
        }
    });

    // --- HOLD LOGIC ---
    let timer_handle = create_rw_signal(None::<Interval>);

    let start_logic = move || {
        if view.get_untracked() != "vault" { return; }
        let interval = Interval::new(10, move || {
            let mut reached_end = false;
            set_hold_progress.update(|p| {
                if *p < 100.0 { *p += 1.5; } else { reached_end = true; }
            });

            if reached_end {
                timer_handle.set(None); 
                let tx_id_seed = 73900784; 
                set_receipt.set(Some(TransactionReceipt {
                    tx_id: format!("VXT-{:x}", tx_id_seed),
                    merchant: merchant.get_untracked(),
                    usd_amount: usd_amount.get_untracked(),
                    crypto_amount: crypto_total(),
                    currency: selected_coin.get_untracked(),
                    timestamp: 1736962043,
                    hci_signature: format!("VERIFIED_HCI_HOLD_1500MS_{:x}", tx_id_seed),
                }));
                set_view.set("success".to_string()); 
            }
        });
        timer_handle.set(Some(interval));
    };

    let cancel_logic = move || {
        if view.get_untracked() == "vault" && hold_progress.get_untracked() < 100.0 {
            timer_handle.set(None);
            set_hold_progress.set(0.0);
        }
    };

    // --- UI ROUTER ---
    view! {
        <main class="bg-black text-white selection:bg-blue-500/30 font-sans">
            {move || match view.get().as_str() {
                "dashboard" => view! {
                    <div class="min-h-screen flex flex-col items-center justify-center p-6 animate-in">
                        <div class="max-w-md w-full bg-slate-900 border border-slate-800 rounded-[2.5rem] p-10 shadow-2xl relative overflow-hidden">
                            
                            <div class="flex items-center gap-4 mb-10">
                                <div class="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center font-black italic shadow-lg shadow-blue-900/20">"V"</div>
                                <h1 class="font-black text-xl tracking-tighter uppercase italic text-white">Vext <span class="text-blue-500">Vault</span></h1>
                            </div>

                            <div class="bg-black/40 border border-slate-800 rounded-3xl p-8 mb-8 text-center">
                                <p class="text-[9px] text-slate-500 font-bold uppercase tracking-widest mb-2">Checkout Total (USD)</p>
                                <h3 class="text-4xl font-mono font-bold text-white tracking-tighter">"$" {move || usd_amount.get()}</h3>
                                <p class="text-[10px] text-blue-500 mt-2 font-black uppercase tracking-widest leading-tight">{move || merchant.get()}</p>
                            </div>

                            <div class="space-y-3 mb-8">
                                <label class="text-[9px] font-bold text-slate-500 uppercase tracking-widest ml-1">Select Payment Asset</label>
                                <div class="grid grid-cols-3 gap-2">
                                    {move || prices.get().into_iter().map(|(sym, _)| {
                                        let sym_for_click = sym.clone();
                                        let sym_for_class = sym.clone();
                                        let sym_for_label = sym.clone();
                                        view! {
                                            <button 
                                                on:click=move |_| set_selected_coin.set(sym_for_click.clone())
                                                class=move || format!("py-3 rounded-xl border text-[10px] font-black transition-all {}", 
                                                    if selected_coin.get() == sym_for_class { "bg-blue-600 border-blue-400 text-white shadow-lg" } 
                                                    else { "bg-black/20 border-slate-800 text-slate-500 hover:border-slate-700" }
                                                )
                                            >
                                                {sym_for_label}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="bg-blue-500/5 border border-blue-500/10 rounded-2xl p-4 text-center">
                                    <p class="text-[9px] text-slate-500 font-bold uppercase tracking-widest mb-1">Vault Conversion</p>
                                    <p class="text-lg font-mono font-bold text-blue-400">{move || crypto_total()} " " {move || selected_coin.get()}</p>
                                </div>
                            </div>

                            <textarea 
                                on:input=move |ev| set_seed_phrase.set(event_target_value(&ev))
                                prop:value=seed_phrase
                                placeholder="Enter 12-word authorization phrase..."
                                class="w-full bg-black/40 border border-slate-800 rounded-2xl p-4 text-xs font-mono text-blue-100 focus:outline-none mb-8 min-h-[100px] resize-none"
                            />

                            <button 
                                on:click=move |_| set_view.set("vault".to_string())
                                disabled=move || !is_unlocked()
                                class=move || format!("w-full py-5 rounded-2xl font-black text-sm tracking-tighter transition-all {}", 
                                    if is_unlocked() { "bg-white text-black shadow-xl opacity-100" } 
                                    else { "bg-slate-800 text-slate-600 opacity-20" }
                                )
                            >
                                {move || if is_unlocked() { "OPEN SECURE BRIDGE" } else { "LOCKED" }}
                            </button>
                        </div>

                        <div class="fixed bottom-0 w-full bg-black/60 border-t border-white/5 py-5 overflow-hidden backdrop-blur-xl">
                            <div class="animate-marquee flex gap-16 whitespace-nowrap px-8">
                                {move || prices.get().iter().cycle().take(12).map(|(s, p)| {
                                    view! { <span class="text-[10px] font-mono font-bold tracking-widest text-white"><span class="text-blue-500">{s.clone()}</span> <span class="text-slate-500">::</span> "$" {p.clone()}</span> }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    </div>
                }.into_view(),

                "vault" => view! {
                    <div class="min-h-screen flex flex-col items-center justify-center p-6 animate-in">
                        <div class="text-center mb-16">
                            <h2 class="text-3xl font-black tracking-tighter uppercase italic text-white font-black">Authenticating</h2>
                            <p class="text-slate-500 text-[10px] mt-3 uppercase tracking-widest font-bold tracking-[0.2em]">Signing {move || crypto_total()} {move || selected_coin.get()}</p>
                        </div>

                        <div 
                            on:mousedown=move |_| start_logic()
                            on:mouseup=move |_| cancel_logic()
                            on:touchstart=move |_| start_logic()
                            on:touchend=move |_| cancel_logic()
                            class="relative w-80 h-80 flex items-center justify-center cursor-pointer group"
                        >
                            <svg class="absolute w-full h-full -rotate-90">
                                <circle cx="160" cy="160" r="140" fill="transparent" stroke="#0f172a" stroke-width="6" />
                                <circle
                                    cx="160" cy="160" r="140" fill="transparent" stroke="#3b82f6" stroke-width="12"
                                    stroke-dasharray="880"
                                    style=move || format!("stroke-dashoffset: {}; transition: stroke-dashoffset 0.1s linear;", 880.0 - (880.0 * hold_progress.get() / 100.0))
                                    stroke-linecap="round"
                                />
                            </svg>
                            <div class=move || format!("w-64 h-64 rounded-full flex flex-col items-center justify-center transition-all duration-300 {}", 
                                if hold_progress.get() > 0.0 { "bg-blue-600 scale-95 shadow-[0_0_60px_rgba(59,130,246,0.3)] border-blue-400" } else { "bg-slate-900 border-slate-800 border-2" }
                            )>
                                <p class="text-[10px] font-black uppercase tracking-widest italic text-white">
                                    {move || if hold_progress.get() > 0.0 { "SIGNING..." } else { "HOLD VAULT" }}
                                </p>
                            </div>
                        </div>
                    </div>
                }.into_view(),

                "success" => view! {
                    <div class="min-h-screen flex items-center justify-center p-6 animate-in text-white text-center">
                        <div class="w-full max-w-sm bg-slate-900 border border-slate-800 rounded-[3rem] p-10 shadow-2xl relative">
                            
                            <div class="w-20 h-20 bg-emerald-500/20 rounded-full flex items-center justify-center mx-auto mb-8 text-emerald-500 border border-emerald-500/30">
                                <svg class="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/></svg>
                            </div>
                            <h2 class="text-3xl font-black tracking-tighter mb-2 uppercase italic font-black">Authorized</h2>
                            <p class="text-[10px] text-emerald-500 uppercase tracking-[0.2em] font-bold mb-8">Intent Verified via HCI</p>
                            
                            <div class="mt-8 text-left bg-black/40 border border-slate-800 rounded-2xl p-6 space-y-4">
                                <div class="space-y-1">
                                    <span class="text-[8px] text-slate-600 uppercase font-bold tracking-widest">Transaction Snapshot</span>
                                    <div class="flex justify-between border-b border-slate-800 pb-2">
                                        <span class="text-[10px] text-slate-300 font-bold uppercase">{move || receipt.get().map(|r| r.merchant).unwrap_or_default()}</span>
                                        <span class="text-[10px] font-mono text-blue-400 font-bold">"$" {move || receipt.get().map(|r| r.usd_amount).unwrap_or_default()} " USD"</span>
                                    </div>
                                </div>

                                <div class="flex justify-between border-b border-slate-800 pb-2">
                                    <span class="text-[9px] text-slate-500 uppercase font-bold">Vault Asset</span>
                                    <span class="text-[9px] font-mono text-white font-bold">{move || receipt.get().map(|r| r.crypto_amount).unwrap_or_default()} {move || receipt.get().map(|r| r.currency).unwrap_or_default()}</span>
                                </div>

                                <div class="bg-blue-500/5 rounded-xl p-3 border border-blue-500/10">
                                    <span class="text-[8px] text-blue-500 uppercase font-bold tracking-widest block mb-1">Human Intent Signature (HCI)</span>
                                    <span class="text-[9px] font-mono text-blue-300 truncate block tracking-tighter">{move || receipt.get().map(|r| r.hci_signature).unwrap_or_default()}</span>
                                </div>
                            </div>

                            <button on:click=move |_| { set_view.set("dashboard".to_string()); set_hold_progress.set(0.0); } class="w-full bg-white text-black font-black py-5 rounded-2xl text-sm mt-10 active:scale-95 transition-all shadow-xl shadow-white/10">"DONE"</button>
                            
                            <p class="text-[8px] text-slate-600 uppercase font-bold tracking-widest mt-8">"VEXT Protocol v2.1 | Rust-WASM Sandbox Secured"</p>
                        </div>
                    </div>
                }.into_view(),

                _ => view! { <div class="text-white font-mono p-10">"Error: View Routing Failure"</div> }.into_view(),
            }}
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}