# SkyrimUncapper MCM Runtime

Modified runtime version of **Skyrim Skill Uncapper SE/AE** used by **Uncapper MCM** to provide per-save runtime configuration through an in-game SkyUI MCM.

This is not the original Skyrim Skill Uncapper repository.

## Features

This runtime adds support for per-save MCM overrides while keeping `SkyrimUncapper.ini` as the baseline configuration.

When runtime overrides are disabled or cleared, Skyrim Skill Uncapper immediately falls back to the values loaded from `SkyrimUncapper.ini`.

Current runtime overrides include:

- Skill Caps
- Skill Formula Caps
- Enchanting settings
- Skill XP multipliers
- Skill XP breakpoints by skill level
- Skill XP breakpoints by character level
- Player Level XP multipliers
- Player Level XP breakpoints by skill level
- Player Level XP breakpoints by character level
- Perks at Level Up
- Attributes at Level Up
- Legendary Skill settings

## Runtime Design

The original INI remains the baseline configuration.

The runtime layer only applies temporary overrides requested by Uncapper MCM.

This allows configuration to be stored per save without rewriting `SkyrimUncapper.ini`.

Clearing the runtime overrides restores the original INI-driven behavior.

Some Skyrim Uncapper systems depend on hooks that are installed only at game startup. Their corresponding `bUse...` options therefore remain controlled by `SkyrimUncapper.ini`.

## Legendary Settings

Legendary runtime configuration supports:

- Legendary skill level threshold
- Skill level after becoming Legendary
- Keeping the current skill level
- Hiding the Legendary button

`bUseLegendarySettings` remains a startup-only INI option because it controls installation of the Legendary hooks.

The original Skyrim Skill Uncapper compatibility constraints for Legendary settings, including the known Custom Skills Framework conflict, remain unchanged.

## Compatibility

This runtime is intended to be used together with **Uncapper MCM**.

It preserves the original Skyrim Skill Uncapper behavior whenever no runtime override is active.

The current runtime is developed and tested for Skyrim AE `1.6.1170`.

## Credits

This project is based on **Skyrim Skill Uncapper SE/AE**.

Original Rust implementation:

- Andrew Spaulding (TheDreadedAndy)

Previous Skyrim Uncapper projects:

- Kassent — Skyrim Uncapper SE
- Vadfromnu — SE/AE update of Kassent's implementation
- Elys — original Skyrim LE Uncapper

MCM runtime override integration:

- Robin

## Original Project

Skyrim Skill Uncapper SE/AE:

https://www.nexusmods.com/skyrimspecialedition/mods/82558

Original repository:

https://github.com/TheDreadedAndy/SkyrimAEUncapper

## License

This project contains modified code from Skyrim Skill Uncapper SE/AE.

Original code:

Copyright 2023 Andrew Spaulding

See `LICENSE.txt` for the full license text.