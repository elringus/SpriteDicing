//! End-to-end tests of the solution crates.

mod common;

use crate::common::*;
use sprite_dicing::{AtlasFormat, Prefs};

#[test]
fn mono_reproduces() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    assert_repro(&MONO, diced, &prefs);
}

#[test]
fn mono_atlas_not_square_when_not_forced() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        atlas_square: false,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    let atlas = bytes_to_img(&diced.atlases[0]);
    assert_ne!(atlas.width(), atlas.height());
}

#[test]
fn mono_atlas_square_when_forced() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        atlas_square: true,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    let atlas = bytes_to_img(&diced.atlases[0]);
    assert_eq!(atlas.width(), atlas.height());
    assert_repro(&MONO, diced, &prefs);
}

#[test]
fn mono_atlas_pot_when_forced() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        atlas_pot: true,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    let atlas = bytes_to_img(&diced.atlases[0]);
    assert_eq!(atlas.width(), 4);
    assert_eq!(atlas.width(), atlas.height());
    assert_repro(&MONO, diced, &prefs);
}

#[test]
fn mono_single_atlases_when_not_limited() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        atlas_size_limit: 1024,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    assert_eq!(diced.atlases.len(), 1);
}

#[test]
fn mono_multiple_atlases_when_limited() {
    let prefs = Prefs {
        unit_size: 1,
        padding: 0,
        atlas_size_limit: 2,
        ..Prefs::default()
    };
    let diced = sprite_dicing::dice_raw(&MONO, &prefs, &AtlasFormat::Png).unwrap();
    assert_eq!(diced.atlases.len(), 2);
    assert_repro(&MONO, diced, &prefs);
}
