//! Runtime overrides for settings controlled by UncapperMCM.

use core::sync::atomic::{
    AtomicU32,
    Ordering,
};

use crate::skyrim::SKILL_COUNT;


// ---------------------------------------------------------
// Constants
// ---------------------------------------------------------

const NO_OVERRIDE: u32 = u32::MAX;

pub const MAX_SKILL_EXP_BREAKPOINTS: usize = 32;
pub const MAX_LEVEL_EXP_BREAKPOINTS: usize = 32;
pub const MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS: usize = 32;
pub const MAX_PERKS_AT_LEVEL_UP_HUNDREDTHS: u32 = 10_000;
pub const ATTRIBUTE_TABLE_COUNT: usize = 12;
pub const MAX_ATTRIBUTE_BREAKPOINTS: usize = 32;

pub const ATTRIBUTE_TABLE_HEALTH_AT_LEVEL_UP: usize = 0;
pub const ATTRIBUTE_TABLE_HEALTH_AT_MAGICKA_LEVEL_UP: usize = 1;
pub const ATTRIBUTE_TABLE_HEALTH_AT_STAMINA_LEVEL_UP: usize = 2;
pub const ATTRIBUTE_TABLE_MAGICKA_AT_HEALTH_LEVEL_UP: usize = 3;
pub const ATTRIBUTE_TABLE_MAGICKA_AT_LEVEL_UP: usize = 4;
pub const ATTRIBUTE_TABLE_MAGICKA_AT_STAMINA_LEVEL_UP: usize = 5;
pub const ATTRIBUTE_TABLE_STAMINA_AT_HEALTH_LEVEL_UP: usize = 6;
pub const ATTRIBUTE_TABLE_STAMINA_AT_MAGICKA_LEVEL_UP: usize = 7;
pub const ATTRIBUTE_TABLE_STAMINA_AT_LEVEL_UP: usize = 8;
pub const ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_HEALTH_LEVEL_UP: usize = 9;
pub const ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_MAGICKA_LEVEL_UP: usize = 10;
pub const ATTRIBUTE_TABLE_CARRY_WEIGHT_AT_STAMINA_LEVEL_UP: usize = 11;


// ---------------------------------------------------------
// Runtime breakpoint table
// ---------------------------------------------------------

struct RuntimeMultiplierTable {
    count: AtomicU32,

    levels: [AtomicU32; MAX_SKILL_EXP_BREAKPOINTS],

    base_hundredths:
        [AtomicU32; MAX_SKILL_EXP_BREAKPOINTS],

    offset_hundredths:
        [AtomicU32; MAX_SKILL_EXP_BREAKPOINTS],
}


impl RuntimeMultiplierTable {
    const fn new() -> Self {
        Self {
            count: AtomicU32::new(NO_OVERRIDE),

            levels:
                [const { AtomicU32::new(0) };
                    MAX_SKILL_EXP_BREAKPOINTS],

            base_hundredths:
                [const { AtomicU32::new(100) };
                    MAX_SKILL_EXP_BREAKPOINTS],

            offset_hundredths:
                [const { AtomicU32::new(100) };
                    MAX_SKILL_EXP_BREAKPOINTS],
        }
    }


    fn clear(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn begin_update(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn set_entry(
        &self,
        index: usize,
        level: u32,
        base_hundredths: u32,
        offset_hundredths: u32,
    ) -> bool {
        if index >= MAX_SKILL_EXP_BREAKPOINTS {
            return false;
        }

        self.levels[index].store(
            level,
            Ordering::Relaxed,
        );

        self.base_hundredths[index].store(
            base_hundredths,
            Ordering::Relaxed,
        );

        self.offset_hundredths[index].store(
            offset_hundredths,
            Ordering::Relaxed,
        );

        true
    }


    fn commit(
        &self,
        count: usize,
    ) -> bool {
        if count == 0 ||
            count > MAX_SKILL_EXP_BREAKPOINTS
        {
            return false;
        }

        let mut previous_level = 0u32;

        for index in 0..count {
            let level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            let base_hundredths =
                self.base_hundredths[index]
                    .load(Ordering::Relaxed);

            let offset_hundredths =
                self.offset_hundredths[index]
                    .load(Ordering::Relaxed);

            if level > 500 {
                return false;
            }

            if index == 0 && level != 0 {
                return false;
            }

            if base_hundredths > 10_000 ||
                offset_hundredths > 10_000
            {
                return false;
            }

            if index > 0 &&
                level <= previous_level
            {
                return false;
            }

            previous_level = level;
        }

        self.count.store(
            count as u32,
            Ordering::Release,
        );

        true
    }


    fn get_nearest(
        &self,
        level: u32,
    ) -> Option<(u32, u32)> {
        let count =
            self.count.load(Ordering::Acquire);

        if count == NO_OVERRIDE {
            return None;
        }

        let count = count as usize;

        if count == 0 ||
            count > MAX_SKILL_EXP_BREAKPOINTS
        {
            return None;
        }

        let mut selected_index = 0usize;
        let mut index = 0usize;

        while index < count {
            let breakpoint_level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            if breakpoint_level <= level {
                selected_index = index;
            } else {
                break;
            }

            index += 1;
        }

        Some((
            self.base_hundredths[selected_index]
                .load(Ordering::Relaxed),

            self.offset_hundredths[selected_index]
                .load(Ordering::Relaxed),
        ))
    }
}


// ---------------------------------------------------------
// Runtime table for simple multipliers.
//
// Used by LevelSkillExpMults.
// Unlike SkillExpGainMults, there is only one multiplier
// per breakpoint.
// ---------------------------------------------------------

struct RuntimeSimpleMultiplierTable {
    count: AtomicU32,

    levels:
        [AtomicU32; MAX_LEVEL_EXP_BREAKPOINTS],

    multiplier_hundredths:
        [AtomicU32; MAX_LEVEL_EXP_BREAKPOINTS],
}


impl RuntimeSimpleMultiplierTable {
    const fn new() -> Self {
        Self {
            count:
                AtomicU32::new(NO_OVERRIDE),

            levels:
                [const { AtomicU32::new(0) };
                    MAX_LEVEL_EXP_BREAKPOINTS],

            multiplier_hundredths:
                [const { AtomicU32::new(100) };
                    MAX_LEVEL_EXP_BREAKPOINTS],
        }
    }


    fn clear(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn begin_update(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn set_entry(
        &self,
        index: usize,
        level: u32,
        multiplier_hundredths: u32,
    ) -> bool {
        if index >= MAX_LEVEL_EXP_BREAKPOINTS {
            return false;
        }

        self.levels[index].store(
            level,
            Ordering::Relaxed,
        );

        self.multiplier_hundredths[index].store(
            multiplier_hundredths,
            Ordering::Relaxed,
        );

        true
    }


    fn commit(
        &self,
        count: usize,
    ) -> bool {
        if count == 0 ||
            count > MAX_LEVEL_EXP_BREAKPOINTS
        {
            return false;
        }

        let mut previous_level = 0u32;

        for index in 0..count {
            let level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            let multiplier =
                self.multiplier_hundredths[index]
                    .load(Ordering::Relaxed);

            if level > 500 {
                return false;
            }

            if index == 0 && level != 0 {
                return false;
            }

            if multiplier > 10_000 {
                return false;
            }

            if index > 0 &&
                level <= previous_level
            {
                return false;
            }

            previous_level = level;
        }

        self.count.store(
            count as u32,
            Ordering::Release,
        );

        true
    }


    fn get_nearest(
        &self,
        level: u32,
    ) -> Option<u32> {
        let count =
            self.count.load(
                Ordering::Acquire
            );

        if count == NO_OVERRIDE {
            return None;
        }

        let count =
            count as usize;

        if count == 0 ||
            count > MAX_LEVEL_EXP_BREAKPOINTS
        {
            return None;
        }

        let mut selected_index = 0usize;
        let mut index = 0usize;

        while index < count {
            let breakpoint_level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            if breakpoint_level <= level {
                selected_index = index;
            } else {
                break;
            }

            index += 1;
        }

        Some(
            self.multiplier_hundredths[selected_index]
                .load(Ordering::Relaxed)
        )
    }
}


// ---------------------------------------------------------
// Runtime table for perks awarded at level up.
// ---------------------------------------------------------

struct RuntimePerksAtLevelUpTable {
    count: AtomicU32,

    levels:
        [AtomicU32; MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS],

    perk_hundredths:
        [AtomicU32; MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS],
}


impl RuntimePerksAtLevelUpTable {
    const fn new() -> Self {
        Self {
            count:
                AtomicU32::new(NO_OVERRIDE),

            levels:
                [const { AtomicU32::new(0) };
                    MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS],

            perk_hundredths:
                [const { AtomicU32::new(100) };
                    MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS],
        }
    }


    fn clear(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn begin_update(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn set_entry(
        &self,
        index: usize,
        level: u32,
        perk_hundredths: u32,
    ) -> bool {
        if index >= MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS ||
            perk_hundredths > MAX_PERKS_AT_LEVEL_UP_HUNDREDTHS
        {
            return false;
        }

        self.levels[index].store(
            level,
            Ordering::Relaxed,
        );

        self.perk_hundredths[index].store(
            perk_hundredths,
            Ordering::Relaxed,
        );

        true
    }


    fn commit(
        &self,
        count: usize,
    ) -> bool {
        if count == 0 ||
            count > MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS
        {
            return false;
        }

        let mut previous_level = 0u32;

        for index in 0..count {
            let level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            let perk_hundredths =
                self.perk_hundredths[index]
                    .load(Ordering::Relaxed);

            if level > 500 {
                return false;
            }

            if perk_hundredths > MAX_PERKS_AT_LEVEL_UP_HUNDREDTHS {
                return false;
            }

            if index == 0 && level != 0 {
                return false;
            }

            if index > 0 &&
                level <= previous_level
            {
                return false;
            }

            previous_level = level;
        }

        self.count.store(
            count as u32,
            Ordering::Release,
        );

        true
    }


    fn get_cumulative_delta(
        &self,
        level: u32,
    ) -> Option<u32> {
        let count =
            self.count.load(Ordering::Acquire);

        if count == NO_OVERRIDE {
            return None;
        }

        let count = count as usize;

        if count == 0 ||
            count > MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS
        {
            return None;
        }

        let target_exclusive =
            level as u64 + 1;

        let mut accumulated_hundredths = 0u64;
        let mut previous_hundredths = 0u64;
        let mut index = 0usize;

        while index < count {
            let breakpoint_level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            if breakpoint_level > level {
                break;
            }

            let next_level =
                if index + 1 < count {
                    self.levels[index + 1]
                        .load(Ordering::Relaxed)
                        as u64
                }
                else {
                    target_exclusive
                };

            let segment_end =
                core::cmp::min(
                    target_exclusive,
                    next_level,
                );

            let perk_hundredths =
                self.perk_hundredths[index]
                    .load(Ordering::Relaxed)
                    as u64;

            accumulated_hundredths +=
                (segment_end - breakpoint_level as u64) *
                perk_hundredths;

            previous_hundredths =
                accumulated_hundredths -
                perk_hundredths;

            index += 1;
        }

        Some(
            ((accumulated_hundredths / 100) -
                (previous_hundredths / 100))
                as u32
        )
    }
}


// ---------------------------------------------------------
// Runtime table for attribute gains at level up.
// ---------------------------------------------------------

struct RuntimeAttributeTable {
    count: AtomicU32,

    levels:
        [AtomicU32; MAX_ATTRIBUTE_BREAKPOINTS],

    values:
        [AtomicU32; MAX_ATTRIBUTE_BREAKPOINTS],
}


impl RuntimeAttributeTable {
    const fn new() -> Self {
        Self {
            count:
                AtomicU32::new(NO_OVERRIDE),

            levels:
                [const { AtomicU32::new(0) };
                    MAX_ATTRIBUTE_BREAKPOINTS],

            values:
                [const { AtomicU32::new(0) };
                    MAX_ATTRIBUTE_BREAKPOINTS],
        }
    }


    fn clear(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn begin_update(&self) {
        self.count.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn set_entry(
        &self,
        index: usize,
        level: u32,
        value: u32,
    ) -> bool {
        if index >= MAX_ATTRIBUTE_BREAKPOINTS ||
            level > 500
        {
            return false;
        }

        self.levels[index].store(
            level,
            Ordering::Relaxed,
        );

        self.values[index].store(
            value,
            Ordering::Relaxed,
        );

        true
    }


    fn commit(
        &self,
        count: usize,
    ) -> bool {
        if count == 0 ||
            count > MAX_ATTRIBUTE_BREAKPOINTS
        {
            return false;
        }

        let mut previous_level = 0u32;

        for index in 0..count {
            let level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            if level > 500 {
                return false;
            }

            if index == 0 && level != 0 {
                return false;
            }

            if index > 0 &&
                level <= previous_level
            {
                return false;
            }

            previous_level = level;
        }

        self.count.store(
            count as u32,
            Ordering::Release,
        );

        true
    }


    fn get_nearest(
        &self,
        level: u32,
    ) -> Option<u32> {
        let count =
            self.count.load(Ordering::Acquire);

        if count == NO_OVERRIDE {
            return None;
        }

        let count = count as usize;

        if count == 0 ||
            count > MAX_ATTRIBUTE_BREAKPOINTS
        {
            return None;
        }

        let mut selected_index = 0usize;
        let mut index = 0usize;

        while index < count {
            let breakpoint_level =
                self.levels[index]
                    .load(Ordering::Relaxed);

            if breakpoint_level <= level {
                selected_index = index;
            } else {
                break;
            }

            index += 1;
        }

        Some(
            self.values[selected_index]
                .load(Ordering::Relaxed)
        )
    }
}


// ---------------------------------------------------------
// Runtime Legendary settings
// ---------------------------------------------------------

#[derive(Copy, Clone)]
pub struct LegendaryRuntimeValues {
    pub keep_skill_level: bool,
    pub hide_button: bool,
    pub skill_level_enable: u32,
    pub skill_level_after: u32,
}


struct RuntimeLegendarySlot {
    packed: AtomicU32,
}


impl RuntimeLegendarySlot {
    const KEEP_SKILL_LEVEL_BIT: u32 = 1 << 0;
    const HIDE_BUTTON_BIT: u32 = 1 << 1;
    const SKILL_LEVEL_ENABLE_SHIFT: u32 = 2;
    const SKILL_LEVEL_AFTER_SHIFT: u32 = 11;
    const SKILL_LEVEL_MASK: u32 = 0x1FF;

    const fn new() -> Self {
        Self {
            packed: AtomicU32::new(0),
        }
    }


    fn store(
        &self,
        values: LegendaryRuntimeValues,
    ) {
        let mut packed =
            (values.skill_level_enable << Self::SKILL_LEVEL_ENABLE_SHIFT) |
            (values.skill_level_after << Self::SKILL_LEVEL_AFTER_SHIFT);

        if values.keep_skill_level {
            packed |= Self::KEEP_SKILL_LEVEL_BIT;
        }

        if values.hide_button {
            packed |= Self::HIDE_BUTTON_BIT;
        }

        self.packed.store(packed, Ordering::Relaxed);
    }


    fn load(&self) -> LegendaryRuntimeValues {
        let packed = self.packed.load(Ordering::Relaxed);

        LegendaryRuntimeValues {
            keep_skill_level:
                packed & Self::KEEP_SKILL_LEVEL_BIT != 0,
            hide_button:
                packed & Self::HIDE_BUTTON_BIT != 0,
            skill_level_enable:
                (packed >> Self::SKILL_LEVEL_ENABLE_SHIFT) & Self::SKILL_LEVEL_MASK,
            skill_level_after:
                (packed >> Self::SKILL_LEVEL_AFTER_SHIFT) & Self::SKILL_LEVEL_MASK,
        }
    }
}


struct RuntimeLegendarySettings {
    active_slot: AtomicU32,
    slots: [RuntimeLegendarySlot; 2],
}


impl RuntimeLegendarySettings {
    const fn new() -> Self {
        Self {
            active_slot: AtomicU32::new(NO_OVERRIDE),
            slots: [const { RuntimeLegendarySlot::new() }; 2],
        }
    }


    fn clear(&self) {
        self.active_slot.store(
            NO_OVERRIDE,
            Ordering::Release,
        );
    }


    fn validate(
        values: LegendaryRuntimeValues,
    ) -> bool {
        if values.skill_level_enable == 0 ||
            values.skill_level_enable > 500 ||
            values.skill_level_after > 500
        {
            return false;
        }

        values.keep_skill_level ||
            values.skill_level_after == 0 ||
            values.skill_level_after < values.skill_level_enable
    }


    fn set(
        &self,
        values: LegendaryRuntimeValues,
    ) -> bool {
        if !Self::validate(values) {
            return false;
        }

        let active_slot = self.active_slot.load(Ordering::Acquire);
        let next_slot = if active_slot == 0 { 1 } else { 0 };

        self.slots[next_slot as usize].store(values);
        self.active_slot.store(next_slot, Ordering::Release);

        true
    }


    fn get(&self) -> Option<LegendaryRuntimeValues> {
        let active_slot = self.active_slot.load(Ordering::Acquire);

        match active_slot {
            0 | 1 => Some(self.slots[active_slot as usize].load()),
            NO_OVERRIDE => None,
            _ => None,
        }
    }
}


// ---------------------------------------------------------
// Skill / Formula Cap overrides
// ---------------------------------------------------------

static SKILL_CAP_OVERRIDES:
    [AtomicU32; SKILL_COUNT] =
    [const { AtomicU32::new(NO_OVERRIDE) };
        SKILL_COUNT];

static FORMULA_CAP_OVERRIDES:
    [AtomicU32; SKILL_COUNT] =
    [const { AtomicU32::new(NO_OVERRIDE) };
        SKILL_COUNT];


// ---------------------------------------------------------
// Enchanting overrides
// ---------------------------------------------------------

static ENCHANT_MAGNITUDE_CAP_OVERRIDE:
    AtomicU32 =
    AtomicU32::new(NO_OVERRIDE);

static ENCHANT_CHARGE_CAP_OVERRIDE:
    AtomicU32 =
    AtomicU32::new(NO_OVERRIDE);

// NO_OVERRIDE = use INI
// 0 = false
// 1 = true
static ENCHANT_USE_LINEAR_CHARGE_OVERRIDE:
    AtomicU32 =
    AtomicU32::new(NO_OVERRIDE);


// ---------------------------------------------------------
// Skill XP base multiplier overrides
// ---------------------------------------------------------

static SKILL_EXP_BASE_MULT_OVERRIDE:
    [AtomicU32; SKILL_COUNT] =
    [const { AtomicU32::new(NO_OVERRIDE) };
        SKILL_COUNT];

static SKILL_EXP_OFFSET_MULT_OVERRIDE:
    [AtomicU32; SKILL_COUNT] =
    [const { AtomicU32::new(NO_OVERRIDE) };
        SKILL_COUNT];


// ---------------------------------------------------------
// Skill XP breakpoint overrides
// ---------------------------------------------------------

static SKILL_EXP_BY_SKILL_LEVEL:
    [RuntimeMultiplierTable; SKILL_COUNT] =
    [const { RuntimeMultiplierTable::new() };
        SKILL_COUNT];

static SKILL_EXP_BY_CHARACTER_LEVEL:
    [RuntimeMultiplierTable; SKILL_COUNT] =
    [const { RuntimeMultiplierTable::new() };
        SKILL_COUNT];

// ---------------------------------------------------------
// Player Level XP multiplier overrides
// ---------------------------------------------------------

static LEVEL_EXP_BASE_MULT_OVERRIDE:
    [AtomicU32; SKILL_COUNT] =
    [const { AtomicU32::new(NO_OVERRIDE) };
        SKILL_COUNT];


static LEVEL_EXP_BY_SKILL_LEVEL:
    [RuntimeSimpleMultiplierTable; SKILL_COUNT] =
    [const { RuntimeSimpleMultiplierTable::new() };
        SKILL_COUNT];


static LEVEL_EXP_BY_CHARACTER_LEVEL:
    [RuntimeSimpleMultiplierTable; SKILL_COUNT] =
    [const { RuntimeSimpleMultiplierTable::new() };
        SKILL_COUNT];


// ---------------------------------------------------------
// Perks at level up override
// ---------------------------------------------------------

static PERKS_AT_LEVEL_UP:
    RuntimePerksAtLevelUpTable =
    RuntimePerksAtLevelUpTable::new();


// ---------------------------------------------------------
// Attribute gains at level up overrides
// ---------------------------------------------------------

static ATTRIBUTE_LEVEL_UP_TABLES:
    [RuntimeAttributeTable; ATTRIBUTE_TABLE_COUNT] =
    [const { RuntimeAttributeTable::new() };
        ATTRIBUTE_TABLE_COUNT];


// ---------------------------------------------------------
// Legendary settings override
// ---------------------------------------------------------

static LEGENDARY_SETTINGS:
    RuntimeLegendarySettings =
    RuntimeLegendarySettings::new();


// ---------------------------------------------------------
// Skill Cap
// ---------------------------------------------------------

pub fn get_skill_cap_override(
    skill_slot: usize,
) -> Option<u32> {
    let value =
        SKILL_CAP_OVERRIDES[skill_slot]
            .load(Ordering::Relaxed);

    if value == NO_OVERRIDE {
        None
    } else {
        Some(value)
    }
}


pub fn set_skill_cap_override(
    skill_slot: usize,
    value: u32,
) -> bool {
    if value == 0 || value > 500 {
        return false;
    }

    SKILL_CAP_OVERRIDES[skill_slot]
        .store(value, Ordering::Relaxed);

    true
}


// ---------------------------------------------------------
// Formula Cap
// ---------------------------------------------------------

pub fn get_formula_cap_override(
    skill_slot: usize,
) -> Option<u32> {
    let value =
        FORMULA_CAP_OVERRIDES[skill_slot]
            .load(Ordering::Relaxed);

    if value == NO_OVERRIDE {
        None
    } else {
        Some(value)
    }
}


pub fn set_formula_cap_override(
    skill_slot: usize,
    value: u32,
) -> bool {
    if value == 0 || value > 500 {
        return false;
    }

    FORMULA_CAP_OVERRIDES[skill_slot]
        .store(value, Ordering::Relaxed);

    true
}


// ---------------------------------------------------------
// Enchanting
// ---------------------------------------------------------

pub fn get_enchant_magnitude_cap_override()
    -> Option<u32>
{
    let value =
        ENCHANT_MAGNITUDE_CAP_OVERRIDE
            .load(Ordering::Relaxed);

    if value == NO_OVERRIDE {
        None
    } else {
        Some(value)
    }
}


pub fn set_enchant_magnitude_cap_override(
    value: u32,
) {
    ENCHANT_MAGNITUDE_CAP_OVERRIDE
        .store(value, Ordering::Relaxed);
}


pub fn get_enchant_charge_cap_override()
    -> Option<u32>
{
    let value =
        ENCHANT_CHARGE_CAP_OVERRIDE
            .load(Ordering::Relaxed);

    if value == NO_OVERRIDE {
        None
    } else {
        Some(value)
    }
}


pub fn set_enchant_charge_cap_override(
    value: u32,
) {
    ENCHANT_CHARGE_CAP_OVERRIDE
        .store(value, Ordering::Relaxed);
}


pub fn get_enchant_use_linear_charge_override()
    -> Option<bool>
{
    let value =
        ENCHANT_USE_LINEAR_CHARGE_OVERRIDE
            .load(Ordering::Relaxed);

    match value {
        NO_OVERRIDE => None,
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}


pub fn set_enchant_use_linear_charge_override(
    value: bool,
) {
    ENCHANT_USE_LINEAR_CHARGE_OVERRIDE.store(
        if value { 1 } else { 0 },
        Ordering::Relaxed,
    );
}


// ---------------------------------------------------------
// Skill XP base multiplier
// ---------------------------------------------------------

pub fn get_skill_exp_base_override(
    skill_slot: usize,
) -> Option<(u32, u32)> {
    let base =
        SKILL_EXP_BASE_MULT_OVERRIDE[skill_slot]
            .load(Ordering::Relaxed);

    let offset =
        SKILL_EXP_OFFSET_MULT_OVERRIDE[skill_slot]
            .load(Ordering::Relaxed);

    if base == NO_OVERRIDE ||
        offset == NO_OVERRIDE
    {
        None
    } else {
        Some((base, offset))
    }
}


pub fn set_skill_exp_base_override(
    skill_slot: usize,
    base_hundredths: u32,
    offset_hundredths: u32,
) {
    SKILL_EXP_BASE_MULT_OVERRIDE[skill_slot]
        .store(
            base_hundredths,
            Ordering::Relaxed,
        );

    SKILL_EXP_OFFSET_MULT_OVERRIDE[skill_slot]
        .store(
            offset_hundredths,
            Ordering::Relaxed,
        );
}


// ---------------------------------------------------------
// Skill XP - by base skill level
// ---------------------------------------------------------

pub fn begin_skill_exp_skill_level_override(
    skill_slot: usize,
) {
    SKILL_EXP_BY_SKILL_LEVEL[skill_slot]
        .begin_update();
}


pub fn set_skill_exp_skill_level_entry(
    skill_slot: usize,
    index: usize,
    level: u32,
    base_hundredths: u32,
    offset_hundredths: u32,
) -> bool {
    SKILL_EXP_BY_SKILL_LEVEL[skill_slot]
        .set_entry(
            index,
            level,
            base_hundredths,
            offset_hundredths,
        )
}


pub fn commit_skill_exp_skill_level_override(
    skill_slot: usize,
    count: usize,
) -> bool {
    SKILL_EXP_BY_SKILL_LEVEL[skill_slot]
        .commit(count)
}


pub fn get_skill_exp_skill_level_override(
    skill_slot: usize,
    level: u32,
) -> Option<(u32, u32)> {
    SKILL_EXP_BY_SKILL_LEVEL[skill_slot]
        .get_nearest(level)
}


// ---------------------------------------------------------
// Skill XP - by character level
// ---------------------------------------------------------

pub fn begin_skill_exp_character_level_override(
    skill_slot: usize,
) {
    SKILL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .begin_update();
}


pub fn set_skill_exp_character_level_entry(
    skill_slot: usize,
    index: usize,
    level: u32,
    base_hundredths: u32,
    offset_hundredths: u32,
) -> bool {
    SKILL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .set_entry(
            index,
            level,
            base_hundredths,
            offset_hundredths,
        )
}


pub fn commit_skill_exp_character_level_override(
    skill_slot: usize,
    count: usize,
) -> bool {
    SKILL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .commit(count)
}


pub fn get_skill_exp_character_level_override(
    skill_slot: usize,
    level: u32,
) -> Option<(u32, u32)> {
    SKILL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .get_nearest(level)
}

// ---------------------------------------------------------
// Player Level XP - base multiplier
// ---------------------------------------------------------

pub fn get_level_exp_base_override(
    skill_slot: usize,
) -> Option<u32> {
    let value =
        LEVEL_EXP_BASE_MULT_OVERRIDE[skill_slot]
            .load(Ordering::Relaxed);

    if value == NO_OVERRIDE {
        None
    } else {
        Some(value)
    }
}


pub fn set_level_exp_base_override(
    skill_slot: usize,
    multiplier_hundredths: u32,
) {
    LEVEL_EXP_BASE_MULT_OVERRIDE[skill_slot]
        .store(
            multiplier_hundredths,
            Ordering::Relaxed,
        );
}


// ---------------------------------------------------------
// Player Level XP - by base skill level
// ---------------------------------------------------------

pub fn begin_level_exp_skill_level_override(
    skill_slot: usize,
) {
    LEVEL_EXP_BY_SKILL_LEVEL[skill_slot]
        .begin_update();
}


pub fn set_level_exp_skill_level_entry(
    skill_slot: usize,
    index: usize,
    level: u32,
    multiplier_hundredths: u32,
) -> bool {
    LEVEL_EXP_BY_SKILL_LEVEL[skill_slot]
        .set_entry(
            index,
            level,
            multiplier_hundredths,
        )
}


pub fn commit_level_exp_skill_level_override(
    skill_slot: usize,
    count: usize,
) -> bool {
    LEVEL_EXP_BY_SKILL_LEVEL[skill_slot]
        .commit(count)
}


pub fn get_level_exp_skill_level_override(
    skill_slot: usize,
    level: u32,
) -> Option<u32> {
    LEVEL_EXP_BY_SKILL_LEVEL[skill_slot]
        .get_nearest(level)
}


// ---------------------------------------------------------
// Player Level XP - by character level
// ---------------------------------------------------------

pub fn begin_level_exp_character_level_override(
    skill_slot: usize,
) {
    LEVEL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .begin_update();
}


pub fn set_level_exp_character_level_entry(
    skill_slot: usize,
    index: usize,
    level: u32,
    multiplier_hundredths: u32,
) -> bool {
    LEVEL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .set_entry(
            index,
            level,
            multiplier_hundredths,
        )
}


pub fn commit_level_exp_character_level_override(
    skill_slot: usize,
    count: usize,
) -> bool {
    LEVEL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .commit(count)
}


pub fn get_level_exp_character_level_override(
    skill_slot: usize,
    level: u32,
) -> Option<u32> {
    LEVEL_EXP_BY_CHARACTER_LEVEL[skill_slot]
        .get_nearest(level)
}


// ---------------------------------------------------------
// Perks at level up
// ---------------------------------------------------------

pub fn begin_perks_at_level_up_override() {
    PERKS_AT_LEVEL_UP.begin_update();
}


pub fn set_perks_at_level_up_entry(
    index: usize,
    level: u32,
    perk_hundredths: u32,
) -> bool {
    PERKS_AT_LEVEL_UP.set_entry(
        index,
        level,
        perk_hundredths,
    )
}


pub fn commit_perks_at_level_up_override(
    count: usize,
) -> bool {
    PERKS_AT_LEVEL_UP.commit(count)
}


pub fn get_perks_at_level_up_cumulative_delta(
    level: u32,
) -> Option<u32> {
    PERKS_AT_LEVEL_UP.get_cumulative_delta(level)
}


// ---------------------------------------------------------
// Attribute gains at level up
// ---------------------------------------------------------

pub fn begin_attribute_override(
    table_index: usize,
) -> bool {
    if table_index >= ATTRIBUTE_TABLE_COUNT {
        return false;
    }

    ATTRIBUTE_LEVEL_UP_TABLES[table_index]
        .begin_update();

    true
}


pub fn set_attribute_entry(
    table_index: usize,
    index: usize,
    level: u32,
    value: u32,
) -> bool {
    if table_index >= ATTRIBUTE_TABLE_COUNT ||
        index >= MAX_ATTRIBUTE_BREAKPOINTS ||
        level > 500
    {
        return false;
    }

    ATTRIBUTE_LEVEL_UP_TABLES[table_index]
        .set_entry(
            index,
            level,
            value,
        )
}


pub fn commit_attribute_override(
    table_index: usize,
    count: usize,
) -> bool {
    if table_index >= ATTRIBUTE_TABLE_COUNT {
        return false;
    }

    ATTRIBUTE_LEVEL_UP_TABLES[table_index]
        .commit(count)
}


pub fn get_attribute_override(
    table_index: usize,
    level: u32,
) -> Option<u32> {
    if table_index >= ATTRIBUTE_TABLE_COUNT {
        return None;
    }

    ATTRIBUTE_LEVEL_UP_TABLES[table_index]
        .get_nearest(level)
}


// ---------------------------------------------------------
// Legendary settings
// ---------------------------------------------------------

pub fn set_legendary_override(
    keep_skill_level: bool,
    hide_button: bool,
    skill_level_enable: u32,
    skill_level_after: u32,
) -> bool {
    LEGENDARY_SETTINGS.set(
        LegendaryRuntimeValues {
            keep_skill_level,
            hide_button,
            skill_level_enable,
            skill_level_after,
        }
    )
}


pub fn get_legendary_override()
    -> Option<LegendaryRuntimeValues>
{
    LEGENDARY_SETTINGS.get()
}


// ---------------------------------------------------------
// Reset
// ---------------------------------------------------------

pub fn clear_overrides() {
    for index in 0..SKILL_COUNT {
        SKILL_CAP_OVERRIDES[index]
            .store(
                NO_OVERRIDE,
                Ordering::Relaxed,
            );

        FORMULA_CAP_OVERRIDES[index]
            .store(
                NO_OVERRIDE,
                Ordering::Relaxed,
            );

        SKILL_EXP_BASE_MULT_OVERRIDE[index]
            .store(
                NO_OVERRIDE,
                Ordering::Relaxed,
            );

        SKILL_EXP_OFFSET_MULT_OVERRIDE[index]
            .store(
                NO_OVERRIDE,
                Ordering::Relaxed,
            );

        SKILL_EXP_BY_SKILL_LEVEL[index]
            .clear();

        SKILL_EXP_BY_CHARACTER_LEVEL[index]
            .clear();

        LEVEL_EXP_BASE_MULT_OVERRIDE[index]
    .store(
        NO_OVERRIDE,
        Ordering::Relaxed,
    );

LEVEL_EXP_BY_SKILL_LEVEL[index]
    .clear();

LEVEL_EXP_BY_CHARACTER_LEVEL[index]
    .clear();
    }

    ENCHANT_MAGNITUDE_CAP_OVERRIDE
        .store(
            NO_OVERRIDE,
            Ordering::Relaxed,
        );

    ENCHANT_CHARGE_CAP_OVERRIDE
        .store(
            NO_OVERRIDE,
            Ordering::Relaxed,
        );

    ENCHANT_USE_LINEAR_CHARGE_OVERRIDE
        .store(
            NO_OVERRIDE,
            Ordering::Relaxed,
        );

    PERKS_AT_LEVEL_UP.clear();

    for index in 0..ATTRIBUTE_TABLE_COUNT {
        ATTRIBUTE_LEVEL_UP_TABLES[index]
            .clear();
    }

    LEGENDARY_SETTINGS.clear();
}
