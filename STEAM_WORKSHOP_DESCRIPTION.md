[h1]Minion and Tower Balance[/h1]

Makes Teamfight Manager 2 lanes less snowbally by weakening minions, strengthening towers and the nexus, and reducing Morgard's temporary minion empowerment buff.

[hr][/hr]

[h2]Default Changes[/h2]

[list]
[*][b]Minion HP:[/b] -10%
[*][b]Minion damage:[/b] -20%
[*][b]Minion HP growth:[/b] unchanged
[*][b]Minion damage growth:[/b] unchanged
[*][b]Tower HP:[/b] +20%
[*][b]Tower damage:[/b] +10%
[*][b]Tower projectile speed:[/b] +10%
[*][b]Nexus HP:[/b] +25%
[*][b]Morgard minion empowerment:[/b] -25%
[/list]

[h2]How To Configure[/h2]

[olist]
[*]Subscribe to the mod and enable it in-game.
[*]Launch or load your save once so the config file exists.
[*]Open [b]balance_config.txt[/b] in the mod folder.
[*]Edit the percentages and save the file.
[*]The mod reloads the config on the next management tick. You can also reload the save or restart the game.
[/olist]

[h3]Config File Location[/h3]

[code]D:\SteamLibrary\steamapps\common\Teamfight Manager2\mods\tfm2-minion-tower-balance\balance_config.txt[/code]

[h3]Config Keys[/h3]

[code]minion_hp=90
minion_damage=80
minion_growth_hp=100
minion_growth_damage=100
tower_hp=120
tower_damage=110
tower_projectile_speed=110
nexus_hp=125
morgard_minion_buff=75[/code]

[h2]Notes[/h2]

[list]
[*][b]100[/b] means vanilla.
[*]Values are clamped between [b]1[/b] and [b]500[/b].
[*]If the config is invalid, the last valid saved/default config stays active.
[*]This is a native Rust code mod, so the game may show the normal code-mod warning when enabling it.
[/list]

[hr][/hr]

[url=https://github.com/eminyilmazz/tfm2-minion-tower-balance]Source code on GitHub[/url]