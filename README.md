# Minion and Tower Balance

Native Rust mod for Teamfight Manager 2 that applies configurable lane-balance values through the SDK's `GameSetting` fields.

## Default Balance

- Melee and ranged minion base HP: `-10%`
- Melee and ranged minion base damage: `-20%`
- Melee and ranged minion HP growth: `0%`
- Melee and ranged minion damage growth: `0%`
- Tower damage to minions: `+10%`
- Tower HP: `+20%`
- Tower attack damage: `+10%`
- Tower projectile speed: `+10%`
- Nexus HP: `+25%`
- Morgard minion empowerment strength: `-25%`

These values are the defaults written to `balance_config.txt`.

## Configuration

Edit this file in the installed mod folder:

```text
your/steam/directory/steamapps/common/Teamfight Manager2/mods/tfm2-minion-tower-balance/balance_config.txt
```

The file uses one `key=value` percentage per line. `100` means vanilla, and values are clamped to `1..=500`.

```text
minion_hp=90
minion_damage=80
minion_growth_hp=100
minion_growth_damage=100
tower_hp=120
tower_damage=110
tower_projectile_speed=110
nexus_hp=125
morgard_minion_buff=75
```

The mod creates the file with defaults if it is missing. It loads the file on server start and reloads it during management ticks when the contents change. If the file is invalid, the last valid saved/default config is kept and the reason is written to `tfm2-minion-tower-balance.log`.

The currently applied values are also stored per save in `mod_save_data` under the `tfm2-minion-tower-balance` namespace so switching values applies as a delta instead of repeatedly compounding.

Saved string keys:

- `config`: desired balance config
- `applied_config`: the config already applied to `GameSetting`

## Implementation Notes

- The patch runs on `ModServerExtension::on_server_start` and mutates `ctx.database.game_setting`.
- Config changes are applied as deltas from `applied_config`, so editing the file does not repeatedly compound stat changes.
- The public SDK exposes `GameSetting` fields for minions, towers, minion wave settings, nexus, and jungle objectives. The Morgard minion buff is `epic_minion_buff_increase`; `morgard_minion_buff` scales that `BuffState`'s numeric strength fields.
- Existing saves that already had earlier versions are migrated by treating the old recommended lane values, and vanilla Morgard buff strength, as the already-applied config.

## Build

Build through the matching SDK. The SDK's batch file can stumble on paths with spaces/trailing slashes, so the PowerShell script is the reliable path:

```powershell
$env:RUSTUP_TOOLCHAIN = 'nightly-2026-06-04-x86_64-pc-windows-msvc'
cd 'your/steam/directory/steamapps/common/Teamfight Manager2/mod-sdk'
.\build_mod_cargo.ps1 -Project '../mods/tfm2-minion-tower-balance' -SdkDir 'your/steam/directory/steamapps/common/Teamfight Manager2/mod-sdk'
```

For Teamfight Manager 2 `0.4.9`, use the `0.4.9` Mod SDK. The SDK should report `base_version.txt` as `0.4.9` and `toolchain_version.txt` as `rustc 1.98.0-nightly (b354133fb 2026-06-03)`.

The DLL mod id must stay equal to the folder name: `tfm2-minion-tower-balance`.

## Steam Workshop

The publish-ready game folder is:

```text
your/steam/directory/steamapps/common/Teamfight Manager2/mods/tfm2-minion-tower-balance
```

It contains `mod.mod_info`, `balance_config.txt`, `thumbnail.png`, `preview.png`, and the compiled `tfm2-minion-tower-balance.dll`. Open `your/steam/directory/steamapps/common/Teamfight Manager2/TFM2ModUploader.exe`, browse to that folder, keep Steam running, choose visibility, enter a change note such as `Initial upload`, and publish.

The uploader can rebuild native Rust code because `mod-sdk` is installed next to the uploader. It skips `src/`, `target/`, `Cargo.toml`, and `Cargo.lock` during upload, so only the runtime DLL and package assets are sent to Workshop.