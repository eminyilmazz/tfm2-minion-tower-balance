Minion and Tower Balance makes Teamfight Manager 2 lanes less snowbally by weakening minions, strengthening towers and the nexus, and reducing Morgard's temporary minion empowerment buff.

Default changes:
- Minion HP: -10%
- Minion damage: -20%
- Minion HP growth: unchanged
- Minion damage growth: unchanged
- Tower HP: +20%
- Tower damage: +10%
- Tower projectile speed: +10%
- Nexus HP: +25%
- Morgard minion empowerment: -25%

How to configure:
1. Subscribe, enable the mod, and launch/load your save once so the config file exists.
2. Open this file:

   D:\SteamLibrary\steamapps\common\Teamfight Manager2\mods\tfm2-minion-tower-balance\balance_config.txt

3. Edit the percentages and save the file.
4. The mod reloads the config on the next management tick while a save is loaded. You can also reload the save or restart the game.

Config keys:
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

Notes:
- `100` means vanilla.
- Values are clamped between `1` and `500`.
- If the config is invalid, the last valid saved/default config stays active.
- This is a native Rust code mod, so the game may show the normal code-mod warning when enabling it.