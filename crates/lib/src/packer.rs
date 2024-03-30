use crate::models::*;
use std::collections::{HashMap, HashSet};

/// Packs diced textures into atlases.
pub(crate) fn pack(diced: Vec<DicedTexture>, prefs: &Prefs) -> Result<Vec<Atlas>> {
    if prefs.uv_inset > 0.5 {
        return Err(Error::Spec("UV inset should be in 0.0 to 0.5 range."));
    }
    if prefs.atlas_size_limit == 0 {
        return Err(Error::Spec("Atlas size limit can't be zero."));
    }
    if prefs.unit_size > prefs.atlas_size_limit {
        return Err(Error::Spec("Unit size can't be above atlas size limit."));
    }

    let total = diced.len();
    let mut atlases = vec![];
    let mut ctx = new_ctx(diced, prefs);
    while !ctx.to_pack.is_empty() {
        Progress::report(prefs, 2, total - ctx.to_pack.len(), total, "Packing units");
        atlases.push(pack_it(&mut ctx)?);
        ctx.packed.clear();
        ctx.units.clear();
    }

    Ok(atlases)
}

struct Context {
    inset: f32,
    square: bool,
    pot: bool,
    size_limit: u32,
    unit_size: u32,
    pad: u32,
    padded_unit_size: u32,
    /// Max. number of units single atlas is able to accommodate.
    unit_capacity: u32,
    /// Total textures left to pack.
    to_pack: Vec<DicedTexture>,
    /// Indexes of to_pack textures packed into current atlas.
    packed: HashSet<usize>,
    /// Units packed into current atlas mapped by hashes.
    units: HashMap<u64, UnitRef>,
}

/// Reference to a diced unit of a diced texture.
struct UnitRef {
    /// Index of the diced texture (via ctx.to_pack) containing referenced unit.
    tex_idx: usize,
    /// Index of the referenced diced unit inside diced texture.
    unit_idx: usize,
}

fn new_ctx(diced: Vec<DicedTexture>, prefs: &Prefs) -> Context {
    let padded_unit_size = prefs.unit_size + prefs.padding * 2;
    let unit_capacity = (prefs.atlas_size_limit / padded_unit_size).pow(2);
    Context {
        inset: prefs.uv_inset,
        square: prefs.atlas_square,
        pot: prefs.atlas_pot,
        size_limit: prefs.atlas_size_limit,
        unit_size: prefs.unit_size,
        pad: prefs.padding,
        padded_unit_size,
        unit_capacity,
        to_pack: diced,
        packed: HashSet::new(),
        units: HashMap::new(),
    }
}

fn pack_it(ctx: &mut Context) -> Result<Atlas> {
    while let Some(tex_idx) = find_packable_texture(ctx) {
        ctx.packed.insert(tex_idx);
        let units = ctx.to_pack[tex_idx].units.iter().enumerate();
        let refs = units.map(|(unit_idx, u)| (u.hash, UnitRef { tex_idx, unit_idx }));
        ctx.units.extend(refs);
    }

    if ctx.packed.is_empty() {
        return Err(Error::Spec(
            "Can't fit single texture; increase atlas size limit.",
        ));
    }

    let atlas_size = eval_atlas_size(ctx);
    let (texture, rects) = bake_atlas(ctx, &atlas_size);
    let packed = extract_packed_textures(ctx);

    Ok(Atlas {
        texture,
        rects,
        packed,
    })
}

fn find_packable_texture(ctx: &Context) -> Option<usize> {
    let mut optimal_texture_idx: Option<usize> = None;
    let mut min_units_to_pack = u32::MAX;

    for (idx, texture) in ctx.to_pack.iter().enumerate() {
        if ctx.packed.contains(&idx) {
            continue;
        }
        let units_to_pack = texture
            .unique
            .iter()
            .filter(|u| !ctx.units.contains_key(u))
            .count() as u32;
        if units_to_pack < min_units_to_pack {
            optimal_texture_idx = Some(idx);
            min_units_to_pack = units_to_pack;
        }
    }

    optimal_texture_idx?;
    if (ctx.units.len() as u32 + min_units_to_pack) <= ctx.unit_capacity {
        optimal_texture_idx
    } else {
        None
    }
}

fn eval_atlas_size(ctx: &Context) -> USize {
    let units_count = ctx.units.len() as u32;
    let size = (units_count as f32).sqrt().ceil() as u32;

    if ctx.pot {
        let size = (size * ctx.padded_unit_size).next_power_of_two();
        return USize::new(size, size);
    }

    if ctx.square {
        let size = size * ctx.padded_unit_size;
        return USize::new(size, size);
    }

    let mut size = USize::new(size, size);
    for width in (1..=size.width).rev() {
        let height = units_count.div_ceil(width);
        if height * ctx.padded_unit_size > ctx.size_limit {
            break;
        }
        if width * height < size.width * size.height {
            size = USize::new(width, height);
        }
    }

    USize::new(
        size.width * ctx.padded_unit_size,
        size.height * ctx.padded_unit_size,
    )
}

fn bake_atlas(ctx: &Context, size: &USize) -> (Texture, HashMap<u64, FRect>) {
    let units_per_row = size.width / ctx.padded_unit_size;
    let mut rects = HashMap::new();
    let mut texture = Texture {
        width: size.width,
        height: size.height,
        pixels: vec![Pixel::default(); (size.width * size.height) as usize],
    };

    // Hash containers in Rust intentionally randomize order for security, while we need
    // stable order to produce identical atlases for identical input, hence the sorting here.
    let mut sorted_hashes = ctx.units.keys().collect::<Vec<_>>();
    sorted_hashes.sort_unstable();

    for (unit_idx, unit_hash) in sorted_hashes.into_iter().enumerate() {
        let unit_ref = &ctx.units[unit_hash];
        let row = unit_idx as u32 / units_per_row;
        let column = unit_idx as u32 % units_per_row;
        let unit = &ctx.to_pack[unit_ref.tex_idx].units[unit_ref.unit_idx];
        set_pixels(ctx, &unit.pixels, column, row, &mut texture);

        let rect = get_uv(ctx, column, row, size);
        let rect = inset_uv(ctx, rect);
        let rect = scale_uv(ctx, rect, unit);
        rects.insert(*unit_hash, rect);
    }

    (texture, rects)
}

fn set_pixels(ctx: &Context, pixels: &[Pixel], column: u32, row: u32, atlas: &mut Texture) {
    let mut from_idx = 0;
    let start_x = column * ctx.padded_unit_size;
    let start_y = row * ctx.padded_unit_size;
    for y in start_y..(start_y + ctx.padded_unit_size) {
        for x in start_x..(start_x + ctx.padded_unit_size) {
            let into_idx = (x + atlas.width * y) as usize;
            atlas.pixels[into_idx] = pixels[from_idx];
            from_idx += 1;
        }
    }
}

fn get_uv(ctx: &Context, column: u32, row: u32, atlas_size: &USize) -> FRect {
    let width = ctx.unit_size as f32 / atlas_size.width as f32;
    let height = ctx.unit_size as f32 / atlas_size.height as f32;
    let x = (column * ctx.padded_unit_size + ctx.pad) as f32 / atlas_size.width as f32;
    let y = (row * ctx.padded_unit_size + ctx.pad) as f32 / atlas_size.height as f32;
    FRect::new(x, y, width, height)
}

fn inset_uv(ctx: &Context, rect: FRect) -> FRect {
    let d = ctx.inset * (rect.width / 2.0);
    let dx2 = d * 2.0;
    FRect::new(rect.x + d, rect.y + d, rect.width - dx2, rect.height - dx2)
}

fn scale_uv(ctx: &Context, rect: FRect, unit: &DicedUnit) -> FRect {
    let mx = unit.rect.width as f32 / ctx.unit_size as f32;
    let my = unit.rect.height as f32 / ctx.unit_size as f32;
    FRect::new(rect.x, rect.y, rect.width * mx, rect.height * my)
}

fn extract_packed_textures(ctx: &mut Context) -> Vec<DicedTexture> {
    let mut packed = Vec::new();
    let mut idx = ctx.to_pack.len() - 1;
    loop {
        if ctx.packed.contains(&idx) {
            packed.push(ctx.to_pack.swap_remove(idx));
        }
        if idx == 0 {
            break;
        }
        idx -= 1;
    }
    packed
}

#[cfg(test)]
mod tests {
    use crate::fixtures::*;
    use crate::models::*;

    #[test]
    fn can_pack_with_defaults() {
        pack(vec![&R1X1, &B1X1], &Prefs::default());
    }

    #[test]
    #[should_panic(expected = "UV inset should be in 0.0 to 0.5 range.")]
    fn errs_when_inset_above_05() {
        let prefs = Prefs {
            uv_inset: 0.85,
            ..defaults()
        };
        pack(vec![&BGRT], &prefs);
    }

    #[test]
    #[should_panic(expected = "Atlas size limit can't be zero.")]
    fn errs_when_limit_is_zero() {
        let prefs = Prefs {
            atlas_size_limit: 0,
            ..defaults()
        };
        pack(vec![&BGRT], &prefs);
    }

    #[test]
    #[should_panic(expected = "Unit size can't be above atlas size limit.")]
    fn errs_when_unit_size_above_limit() {
        let prefs = Prefs {
            unit_size: 2,
            atlas_size_limit: 1,
            ..defaults()
        };
        pack(vec![&BGRT], &prefs);
    }

    #[test]
    fn when_empty_input_empty_vec_is_returned() {
        assert_eq!(pack(vec![], &Prefs::default()).len(), 0);
    }

    #[test]
    fn when_content_doesnt_fit_multiple_atlases_are_produced() {
        let prefs = Prefs {
            atlas_size_limit: 1,
            ..defaults()
        };
        assert_eq!(pack(vec![&B1X1, &R1X1], &prefs).len(), 2);
    }

    #[test]
    #[should_panic(expected = "Can't fit single texture; increase atlas size limit.")]
    fn errs_when_content_from_single_texture_doesnt_fit() {
        let prefs = Prefs {
            atlas_size_limit: 1,
            ..defaults()
        };
        pack(vec![&BGRT], &prefs);
    }

    #[test]
    fn when_square_is_optimal_atlas_is_square() {
        let prefs = Prefs {
            atlas_size_limit: 4,
            ..defaults()
        };
        let atlas = pack(vec![&RGBY, &B1X1], &prefs).pop().unwrap();
        assert_eq!(atlas.texture.width, 2);
        assert_eq!(atlas.texture.height, 2);
    }

    #[test]
    fn when_square_is_not_optimal_atlas_is_not_square() {
        let prefs = Prefs {
            atlas_size_limit: 4,
            ..defaults()
        };
        let atlas = pack(vec![&RGBY, &C1X1], &prefs).pop().unwrap();
        assert_eq!(atlas.texture.width, 3);
        assert_eq!(atlas.texture.height, 2);
    }

    #[test]
    fn when_square_is_not_optimal_but_forced_atlas_is_square() {
        let prefs = Prefs {
            atlas_size_limit: 4,
            atlas_square: true,
            ..defaults()
        };
        let atlas = pack(vec![&RGBY, &C1X1], &prefs).pop().unwrap();
        assert_eq!(atlas.texture.width, 3);
        assert_eq!(atlas.texture.height, 3);
    }

    #[test]
    fn when_pot_forced_atlas_is_power_of_two() {
        let prefs = Prefs {
            atlas_size_limit: 4,
            atlas_pot: true,
            ..defaults()
        };
        let atlas = pack(vec![&RGBY, &C1X1], &prefs).pop().unwrap();
        assert_eq!(atlas.texture.width, 4);
        assert_eq!(atlas.texture.height, 4);
    }

    #[test]
    fn unused_pixels_are_clear() {
        let prefs = Prefs {
            atlas_size_limit: 4,
            atlas_pot: true,
            ..defaults()
        };
        let atlas = pack(vec![&RGBY, &C1X1], &prefs).pop().unwrap();
        let clear = atlas.texture.pixels.into_iter().filter(|p| p.eq(&T));
        assert_eq!(clear.count(), 11);
    }

    #[test]
    fn uvs_are_mapped() {
        let atlas = pack(vec![&R1X1], &defaults()).pop().unwrap();
        let rect = atlas.rects.values().next().unwrap();
        assert_eq!(*rect, FRect::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn inset_uvs_are_scaled() {
        let prefs = Prefs {
            uv_inset: 0.2,
            ..defaults()
        };
        let atlas = pack(vec![&Y1X1], &prefs).pop().unwrap();
        let rect = atlas.rects.values().next().unwrap();
        assert_eq!(*rect, FRect::new(0.1, 0.1, 0.8, 0.8));
    }

    #[test]
    fn overflow_uvs_are_cropped() {
        let prefs = Prefs {
            unit_size: 2,
            padding: 1,
            ..defaults()
        };
        let atlas = pack(vec![&M1X1], &prefs).pop().unwrap();
        let rect = atlas.rects.values().next().unwrap();
        assert_eq!(*rect, FRect::new(0.25, 0.25, 0.25, 0.25));
    }

    #[test]
    fn reports_progress() {
        let progress = sample_progress(|p| drop(pack(vec![&M1X1], &p)));
        assert_eq!(progress.ratio, 0.6);
    }

    fn pack(src: Vec<&dyn AnySource>, prefs: &Prefs) -> Vec<Atlas> {
        let sprites = src.into_iter().map(|s| s.sprite()).collect::<Vec<_>>();
        let diced = crate::dicer::dice(&sprites, prefs).unwrap();
        crate::packer::pack(diced, prefs).unwrap()
    }

    fn defaults() -> Prefs {
        Prefs {
            unit_size: 1,
            padding: 0,
            ..Prefs::default()
        }
    }
}
