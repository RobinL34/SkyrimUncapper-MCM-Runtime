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
        if index >= MAX_PERKS_AT_LEVEL_UP_BREAKPOINTS {
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
) {
    SKILL_CAP_OVERRIDES[skill_slot]
        .store(value, Ordering::Relaxed);
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
) {
    FORMULA_CAP_OVERRIDES[skill_slot]
        .store(value, Ordering::Relaxed);
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
}
