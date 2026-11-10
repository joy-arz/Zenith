//! Horizon: native Rust client for Minecraft Java Edition 26.3 gameplay.
#![forbid(unsafe_code)]

#[cfg(not(any(feature = "system-allocator", feature = "alloc-census")))]
#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::ALLOCATOR;
#[cfg(all(feature = "alloc-census", not(feature = "system-allocator")))]
#[global_allocator]
static ALLOCATOR: horizon_threads::CensusAllocator = horizon_threads::CensusAllocator;

/// The mod API level this game serves, kept in the program for the launcher
/// to read without running it (`horizon_mods::api_level::MARK`). `main`
/// reads it, so the linker keeps it.
static MOD_API_LEVEL: [u8; 27] = horizon_mods::api_level::MARK;

fn main() {
    std::hint::black_box(&MOD_API_LEVEL);
    // Zenith ships this executable. Register the extras that moved out of the engine.
    #[cfg(feature = "friends")]
    horizon_hooks::register(&horizon_friends::game::HOOKS)
        .expect("register friends before startup");
    #[cfg(feature = "horizon-extras")]
    horizon_hooks::register(&horizon_official_servers::HOOKS)
        .expect("register servers before startup");
    #[cfg(feature = "discord")]
    {
        horizon_hooks::register(&horizon_discord::HOOKS).expect("register Discord before startup");
        horizon_hooks::settings::register(&horizon_discord::SETTING)
            .expect("register Discord settings before startup");
        horizon_hooks::pages::register(&horizon_discord::SECTION)
            .expect("register Discord page before startup");
    }
    horizon_client::entry::main();
}
