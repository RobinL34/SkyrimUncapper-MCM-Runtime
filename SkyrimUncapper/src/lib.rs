//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Kassent
//! @author Vadfromnu
//! @brief Top level library configuration and initialization.
//! @bug No known bugs.
//!

// Our crate name is stupid, for historical reasons.
#![allow(non_snake_case)]

#![no_std]
extern crate alloc;

mod skyrim;
mod hooks;
mod settings;
mod runtime_settings;

// For macros.
pub use core;

use core::ffi::CStr;
use alloc::alloc::{GlobalAlloc, Layout};

use libskyrim::log::{skse_message, skse_fatal};
use libskyrim::version::{SkseVersion, PACKED_SKSE_VERSION, CURRENT_RELEASE_RUNTIME};
use libskyrim::plugin_api::{SksePluginVersionData, SkseInterface};
use libskyrim::patcher::flatten_patch_groups;

use skyrim::{GAME_SIGNATURES, NUM_GAME_SIGNATURES};
use hooks::{HOOK_SIGNATURES, NUM_HOOK_SIGNATURES};

////////////////////////////////////////////////////////////////////////////////////////////////////

// Since we're in a no_std environment, we need to define a memory allocator for the alloc crate to
// use.
struct SystemAlloc;

// These are defined in CRT, but not in libc.
extern "C" {
    fn _aligned_malloc(size: usize, align: usize) -> *mut u8;
    fn _aligned_free(ptr: *mut u8);
}

unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(
        &self,
        layout: Layout
    ) -> *mut u8 {
        _aligned_malloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(
        &self,
        ptr: *mut u8,
        _layout: Layout
    ) {
        _aligned_free(ptr);
    }
}

#[global_allocator]
static A: SystemAlloc = SystemAlloc;

////////////////////////////////////////////////////////////////////////////////////////////////////

const NUM_PATCHES: usize = NUM_GAME_SIGNATURES + NUM_HOOK_SIGNATURES;

libskyrim::plugin_api::plugin_version_data! {
    author: "Andrew Spaulding (Kasplat)",
    email: "andyespaulding@gmail.com",
    version_indep_ex: SksePluginVersionData::VINDEPEX_NO_STRUCT_USE,
    version_indep: SksePluginVersionData::VINDEP_ADDRESS_LIBRARY_POST_AE,
    compat_versions: []
}

///
/// Plugin entry point.
///
/// Called by the SKSE64 crate when our plugin is loaded. This function will only be called once.
///
#[no_mangle]
pub fn skse_plugin_rust_entry(
    skse: &SkseInterface
) -> Result<(), ()> {
    // Log runtime/skse info.
    skse_message!(
        "{} {:?} ({})\n\
         Compiled: SKSE64 {}, Skyrim SE {}\n\
         Running: SKSE64 {}, Skyrim SE {}\n\
         Base addr: {:#x}",
        unsafe { CStr::from_ptr(SKSEPlugin_Version.name.as_ptr()).to_str().unwrap() },
        SKSEPlugin_Version.plugin_version,
        env!("UNCAPPER_GIT_VERSION"),
        PACKED_SKSE_VERSION,
        CURRENT_RELEASE_RUNTIME,
        (*skse).skse_version.unwrap(),
        (*skse).runtime_version.unwrap(),
        libskyrim::reloc::RelocAddr::base()
    );

    settings::init(core_util::cstr!("Data\\SKSE\\Plugins\\SkyrimUncapper.ini"));

    let patches = flatten_patch_groups::<NUM_PATCHES>(&[&GAME_SIGNATURES, &HOOK_SIGNATURES]);
    if let Err(_) = libskyrim::patcher::apply(patches) {
        skse_fatal!(
            "Failed to install the requested set of game patches. See log for details.\n\
             It is safe to continue playing; none of this mods changes have been applied."
        );
        return Err(());
    }

    skse_message!("Initialization complete!");
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Runtime override API exposed to UncapperMCMBridge
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_SetSkillCapOverride(
    skill_slot: u32,
    value: u32,
) -> bool {
    let skill_slot = skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::set_skill_cap_override(
        skill_slot,
        value,
    );

    true
}

#[no_mangle]
pub extern "system" fn Uncapper_SetFormulaCapOverride(
    skill_slot: u32,
    value: u32,
) -> bool {
    let skill_slot = skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::set_formula_cap_override(
        skill_slot,
        value,
    );

    true
}

#[no_mangle]
pub extern "system" fn Uncapper_ClearOverrides() {
    runtime_settings::clear_overrides();
}

fn skill_from_slot(
    skill_slot: u32,
) -> Option<skyrim::ActorAttribute> {
    match skill_slot {
        0 => Some(skyrim::ActorAttribute::OneHanded),
        1 => Some(skyrim::ActorAttribute::TwoHanded),
        2 => Some(skyrim::ActorAttribute::Marksman),
        3 => Some(skyrim::ActorAttribute::Block),
        4 => Some(skyrim::ActorAttribute::Smithing),
        5 => Some(skyrim::ActorAttribute::HeavyArmor),
        6 => Some(skyrim::ActorAttribute::LightArmor),
        7 => Some(skyrim::ActorAttribute::Pickpocket),
        8 => Some(skyrim::ActorAttribute::LockPicking),
        9 => Some(skyrim::ActorAttribute::Sneak),
        10 => Some(skyrim::ActorAttribute::Alchemy),
        11 => Some(skyrim::ActorAttribute::Speechcraft),
        12 => Some(skyrim::ActorAttribute::Alteration),
        13 => Some(skyrim::ActorAttribute::Conjuration),
        14 => Some(skyrim::ActorAttribute::Destruction),
        15 => Some(skyrim::ActorAttribute::Illusion),
        16 => Some(skyrim::ActorAttribute::Restoration),
        17 => Some(skyrim::ActorAttribute::Enchanting),
        _ => None,
    }
}

#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillCap(
    skill_slot: u32,
) -> u32 {
    let Some(skill) = skill_from_slot(skill_slot) else {
        return u32::MAX;
    };

    settings::SETTINGS
        .skill_caps
        .get(skill)
        .get()
}

#[no_mangle]
pub extern "system" fn Uncapper_GetIniFormulaCap(
    skill_slot: u32,
) -> u32 {
    let Some(skill) = skill_from_slot(skill_slot) else {
        return u32::MAX;
    };

    settings::SETTINGS
        .skill_formula_caps
        .get(skill)
        .get()
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Enchanting runtime override API
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_SetEnchantMagnitudeCapOverride(
    value: u32,
) -> bool {
    if value < 1 || value > 500 {
        return false;
    }

    runtime_settings::set_enchant_magnitude_cap_override(
        value,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetEnchantChargeCapOverride(
    value: u32,
) -> bool {
    if value < 1 || value > 199 {
        return false;
    }

    runtime_settings::set_enchant_charge_cap_override(
        value,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetEnchantUseLinearChargeOverride(
    value: u32,
) -> bool {
    let enabled = match value {
        0 => false,
        1 => true,
        _ => return false,
    };

    runtime_settings::set_enchant_use_linear_charge_override(
        enabled,
    );

    true
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Enchanting INI getters
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniEnchantMagnitudeCap() -> u32 {
    settings::SETTINGS
        .enchant
        .magnitude_cap
        .get()
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniEnchantChargeCap() -> u32 {
    settings::SETTINGS
        .enchant
        .charge_cap
        .get()
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniEnchantUseLinearCharge() -> u32 {
    if settings::SETTINGS
        .enchant
        .use_linear_charge
        .get()
    {
        1
    } else {
        0
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP multiplier API
////////////////////////////////////////////////////////////////////////////////////////////////////

const MAX_MULTIPLIER_HUNDREDTHS: u32 = 10_000;
const MAX_BREAKPOINT_LEVEL: u32 = 500;


fn multiplier_to_hundredths(
    value: f32,
) -> u32 {
    if !value.is_finite() ||
        value < 0.0 ||
        value > 100.0
    {
        return u32::MAX;
    }

    (value * 100.0 + 0.5) as u32
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - INI base multiplier getters
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpBaseMultiplier(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let multiplier =
        settings::SETTINGS
            .skill_exp_mults
            .get(skill)
            .get();

    multiplier_to_hundredths(
        multiplier.base,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpOffsetMultiplier(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let multiplier =
        settings::SETTINGS
            .skill_exp_mults
            .get(skill)
            .get();

    multiplier_to_hundredths(
        multiplier.offset,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - runtime base multiplier override
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_SetSkillExpBaseOverride(
    skill_slot: u32,
    base_hundredths: u32,
    offset_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if base_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS ||
        offset_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_skill_exp_base_override(
        skill_slot,
        base_hundredths,
        offset_hundredths,
    );

    true
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - INI breakpoints by base skill level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpSkillLevelBreakpointCount(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_skills
            .get(skill);

    section.len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpSkillLevelBreakpointLevel(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_skills
            .get(skill);

    let Some((level, _)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpSkillLevelBreakpointBaseMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_skills
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier.base,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpSkillLevelBreakpointOffsetMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_skills
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier.offset,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - runtime breakpoints by base skill level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_BeginSkillExpSkillLevelOverride(
    skill_slot: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::begin_skill_exp_skill_level_override(
        skill_slot,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetSkillExpSkillLevelBreakpoint(
    skill_slot: u32,
    index: u32,
    level: u32,
    base_hundredths: u32,
    offset_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let index =
        index as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if index >=
        runtime_settings::MAX_SKILL_EXP_BREAKPOINTS
    {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    if base_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS ||
        offset_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_skill_exp_skill_level_entry(
        skill_slot,
        index,
        level,
        base_hundredths,
        offset_hundredths,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitSkillExpSkillLevelOverride(
    skill_slot: u32,
    count: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let count =
        count as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if count == 0 ||
        count >
            runtime_settings::MAX_SKILL_EXP_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_skill_exp_skill_level_override(
        skill_slot,
        count,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - INI breakpoints by character level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpCharacterLevelBreakpointCount(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_pc_lvl
            .get(skill);

    section.len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpCharacterLevelBreakpointLevel(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_pc_lvl
            .get(skill);

    let Some((level, _)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpCharacterLevelBreakpointBaseMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_pc_lvl
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier.base,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniSkillExpCharacterLevelBreakpointOffsetMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .skill_exp_mults_with_pc_lvl
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier.offset,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Skill XP - runtime breakpoints by character level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_BeginSkillExpCharacterLevelOverride(
    skill_slot: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::begin_skill_exp_character_level_override(
        skill_slot,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetSkillExpCharacterLevelBreakpoint(
    skill_slot: u32,
    index: u32,
    level: u32,
    base_hundredths: u32,
    offset_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let index =
        index as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if index >=
        runtime_settings::MAX_SKILL_EXP_BREAKPOINTS
    {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    if base_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS ||
        offset_hundredths >
            MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_skill_exp_character_level_entry(
        skill_slot,
        index,
        level,
        base_hundredths,
        offset_hundredths,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitSkillExpCharacterLevelOverride(
    skill_slot: u32,
    count: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let count =
        count as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if count == 0 ||
        count >
            runtime_settings::MAX_SKILL_EXP_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_skill_exp_character_level_override(
        skill_slot,
        count,
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP multiplier API
////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - INI base multiplier
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpMultiplier(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let multiplier =
        settings::SETTINGS
            .level_exp_mults
            .get(skill)
            .get();

    multiplier_to_hundredths(
        multiplier,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - runtime base multiplier
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_SetLevelExpMultiplierOverride(
    skill_slot: u32,
    multiplier_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if multiplier_hundredths >
        MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_level_exp_base_override(
        skill_slot,
        multiplier_hundredths,
    );

    true
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - INI breakpoints by base skill level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpSkillLevelBreakpointCount(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_skills
            .get(skill);

    section.len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpSkillLevelBreakpointLevel(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_skills
            .get(skill);

    let Some((level, _)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpSkillLevelBreakpointMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_skills
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - runtime breakpoints by base skill level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_BeginLevelExpSkillLevelOverride(
    skill_slot: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::begin_level_exp_skill_level_override(
        skill_slot,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetLevelExpSkillLevelBreakpoint(
    skill_slot: u32,
    index: u32,
    level: u32,
    multiplier_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let index =
        index as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if index >=
        runtime_settings::MAX_LEVEL_EXP_BREAKPOINTS
    {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    if multiplier_hundredths >
        MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_level_exp_skill_level_entry(
        skill_slot,
        index,
        level,
        multiplier_hundredths,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitLevelExpSkillLevelOverride(
    skill_slot: u32,
    count: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let count =
        count as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if count == 0 ||
        count >
            runtime_settings::MAX_LEVEL_EXP_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_level_exp_skill_level_override(
        skill_slot,
        count,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - INI breakpoints by character level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpCharacterLevelBreakpointCount(
    skill_slot: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_pc_lvl
            .get(skill);

    section.len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpCharacterLevelBreakpointLevel(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_pc_lvl
            .get(skill);

    let Some((level, _)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniLevelExpCharacterLevelBreakpointMultiplier(
    skill_slot: u32,
    index: u32,
) -> u32 {
    let Some(skill) =
        skill_from_slot(skill_slot)
    else {
        return u32::MAX;
    };

    let section =
        settings::SETTINGS
            .level_exp_mults_with_pc_lvl
            .get(skill);

    let Some((_, multiplier)) =
        section.get_at(index as usize)
    else {
        return u32::MAX;
    };

    multiplier_to_hundredths(
        multiplier,
    )
}


////////////////////////////////////////////////////////////////////////////////////////////////////
// Player Level XP - runtime breakpoints by character level
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_BeginLevelExpCharacterLevelOverride(
    skill_slot: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    runtime_settings::begin_level_exp_character_level_override(
        skill_slot,
    );

    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetLevelExpCharacterLevelBreakpoint(
    skill_slot: u32,
    index: u32,
    level: u32,
    multiplier_hundredths: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let index =
        index as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if index >=
        runtime_settings::MAX_LEVEL_EXP_BREAKPOINTS
    {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    if multiplier_hundredths >
        MAX_MULTIPLIER_HUNDREDTHS
    {
        return false;
    }

    runtime_settings::set_level_exp_character_level_entry(
        skill_slot,
        index,
        level,
        multiplier_hundredths,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitLevelExpCharacterLevelOverride(
    skill_slot: u32,
    count: u32,
) -> bool {
    let skill_slot =
        skill_slot as usize;

    let count =
        count as usize;

    if skill_slot >= skyrim::SKILL_COUNT {
        return false;
    }

    if count == 0 ||
        count >
            runtime_settings::MAX_LEVEL_EXP_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_level_exp_character_level_override(
        skill_slot,
        count,
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Perks at Level Up API
////////////////////////////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub extern "system" fn Uncapper_GetIniPerksAtLevelUpBreakpointCount() -> u32 {
    settings::SETTINGS
        .perks_at_lvl_up
        .len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniPerksAtLevelUpBreakpointLevel(
    index: u32,
) -> u32 {
    let Some((level, _)) =
        settings::SETTINGS
            .perks_at_lvl_up
            .get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniPerksAtLevelUpBreakpointValue(
    index: u32,
) -> u32 {
    let Some((_, value)) =
        settings::SETTINGS
            .perks_at_lvl_up
            .get_at(index as usize)
    else {
        return u32::MAX;
    };

    if !value.is_finite() ||
        value < 0.0 ||
        value > (u32::MAX as f32 / 100.0)
    {
        return u32::MAX;
    }

    (value * 100.0 + 0.5) as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_BeginPerksAtLevelUpOverride() -> bool {
    runtime_settings::begin_perks_at_level_up_override();
    true
}


#[no_mangle]
pub extern "system" fn Uncapper_SetPerksAtLevelUpBreakpoint(
    index: u32,
    level: u32,
    perk_hundredths: u32,
) -> bool {
    let index = index as usize;

    if index >=
        runtime_settings::MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS
    {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    runtime_settings::set_perks_at_level_up_entry(
        index,
        level,
        perk_hundredths,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitPerksAtLevelUpOverride(
    count: u32,
) -> bool {
    let count = count as usize;

    if count == 0 ||
        count >
            runtime_settings::MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_perks_at_level_up_override(
        count,
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Attributes at Level Up API
////////////////////////////////////////////////////////////////////////////////////////////////////

fn ini_attribute_table(
    table_index: usize,
) -> Option<&'static settings::LeveledIniSection<u32>> {
    match table_index {
        runtime_settings::ATTRIBUTE_TABLE_HEALTH_AT_LEVEL_UP =>
            Some(&settings::SETTINGS.hp_at_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_HEALTH_AT_MAGICKA_LEVEL_UP =>
            Some(&settings::SETTINGS.hp_at_mp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_HEALTH_AT_STAMINA_LEVEL_UP =>
            Some(&settings::SETTINGS.hp_at_sp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_MAGICKA_AT_HEALTH_LEVEL_UP =>
            Some(&settings::SETTINGS.mp_at_hp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_MAGICKA_AT_LEVEL_UP =>
            Some(&settings::SETTINGS.mp_at_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_MAGICKA_AT_STAMINA_LEVEL_UP =>
            Some(&settings::SETTINGS.mp_at_sp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_STAMINA_AT_HEALTH_LEVEL_UP =>
            Some(&settings::SETTINGS.sp_at_hp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_STAMINA_AT_MAGICKA_LEVEL_UP =>
            Some(&settings::SETTINGS.sp_at_mp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_STAMINA_AT_LEVEL_UP =>
            Some(&settings::SETTINGS.sp_at_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_HEALTH_LEVEL_UP =>
            Some(&settings::SETTINGS.cw_at_hp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_MAGICKA_LEVEL_UP =>
            Some(&settings::SETTINGS.cw_at_mp_lvl_up),
        runtime_settings::ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_STAMINA_LEVEL_UP =>
            Some(&settings::SETTINGS.cw_at_sp_lvl_up),
        _ => None,
    }
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniAttributeBreakpointCount(
    table_index: u32,
) -> u32 {
    let Some(table) =
        ini_attribute_table(table_index as usize)
    else {
        return u32::MAX;
    };

    table.len() as u32
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniAttributeBreakpointLevel(
    table_index: u32,
    index: u32,
) -> u32 {
    let Some(table) =
        ini_attribute_table(table_index as usize)
    else {
        return u32::MAX;
    };

    let Some((level, _)) =
        table.get_at(index as usize)
    else {
        return u32::MAX;
    };

    level
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniAttributeBreakpointValue(
    table_index: u32,
    index: u32,
) -> u32 {
    let Some(table) =
        ini_attribute_table(table_index as usize)
    else {
        return u32::MAX;
    };

    let Some((_, value)) =
        table.get_at(index as usize)
    else {
        return u32::MAX;
    };

    value
}


#[no_mangle]
pub extern "system" fn Uncapper_BeginAttributeOverride(
    table_index: u32,
) -> bool {
    let table_index = table_index as usize;

    if table_index >= runtime_settings::ATTRIBUTE_TABLE_COUNT {
        return false;
    }

    runtime_settings::begin_attribute_override(
        table_index,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_SetAttributeBreakpoint(
    table_index: u32,
    index: u32,
    level: u32,
    value: u32,
) -> bool {
    let table_index = table_index as usize;
    let index = index as usize;

    if table_index >= runtime_settings::ATTRIBUTE_TABLE_COUNT {
        return false;
    }

    if index >= runtime_settings::MAX_ATTRIBUTE_BREAKPOINTS {
        return false;
    }

    if level > MAX_BREAKPOINT_LEVEL {
        return false;
    }

    runtime_settings::set_attribute_entry(
        table_index,
        index,
        level,
        value,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_CommitAttributeOverride(
    table_index: u32,
    count: u32,
) -> bool {
    let table_index = table_index as usize;
    let count = count as usize;

    if table_index >= runtime_settings::ATTRIBUTE_TABLE_COUNT {
        return false;
    }

    if count == 0 ||
        count > runtime_settings::MAX_ATTRIBUTE_BREAKPOINTS
    {
        return false;
    }

    runtime_settings::commit_attribute_override(
        table_index,
        count,
    )
}


#[no_mangle]
pub extern "system" fn Uncapper_GetIniUseAttributesAtLevelUp() -> u32 {
    if settings::SETTINGS
        .general
        .attr_points_en
        .get()
    {
        1
    } else {
        0
    }
}
