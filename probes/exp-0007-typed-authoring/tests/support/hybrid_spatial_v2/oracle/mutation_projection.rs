use super::mutations::Controls;
use super::types::{Aabb, Log, PaintReference};

pub(super) fn projection_controls(controls: &mut Controls<'_>) {
    mapping(controls);
    geometry(controls);
    clips(controls);
    paints(controls);
    items(controls, false);
    items(controls, true);
}

fn mapping(controls: &mut Controls<'_>) {
    controls.check("mapping order", |log| {
        log.observations[0].projection.mapping.swap(0, 1);
    });
    controls.check("mapping key", |log| {
        log.observations[0].projection.mapping[0].0 += 1;
    });
    controls.check("sentinel mapping", |log| {
        log.observations[0].projection.mapping[0].1 = Some("root".to_owned());
    });
    controls.check("mapped path", |log| {
        log.observations[0].projection.mapping[1]
            .1
            .as_mut()
            .expect("root should map")
            .push('x');
    });
    controls.check("missing mapping", |log| {
        log.observations[0].projection.mapping.pop();
    });
    controls.check("duplicate mapping", |log| {
        let duplicate = log.observations[0].projection.mapping[1].clone();
        log.observations[0].projection.mapping.push(duplicate);
    });
}

fn geometry(controls: &mut Controls<'_>) {
    controls.check("geometry order", |log| {
        log.observations[0].projection.geometry.swap(0, 1);
    });
    controls.check("geometry removed row", |log| {
        log.observations[0].projection.geometry.pop();
    });
    controls.check("geometry duplicate row", |log| {
        let row = log.observations[0].projection.geometry[1].clone();
        log.observations[0].projection.geometry.push(row);
    });
    for (label, field) in [
        ("geometry key", 0_u8),
        ("geometry path", 1),
        ("geometry base x", 2),
        ("geometry base y", 3),
        ("geometry base width", 4),
        ("geometry base height", 5),
        ("geometry affine a", 6),
        ("geometry affine b", 7),
        ("geometry affine c", 8),
        ("geometry affine d", 9),
        ("geometry affine tx", 10),
        ("geometry affine ty", 11),
        ("geometry determinant", 12),
        ("geometry aabb empty", 13),
        ("geometry aabb min x", 14),
        ("geometry aabb min y", 15),
        ("geometry aabb max x", 16),
        ("geometry aabb max y", 17),
    ] {
        controls.check(label, move |log| {
            let row = &mut log.observations[0].projection.geometry[1];
            match field {
                0 => row.key += 1,
                1 => row.path.as_mut().expect("root should map").push('x'),
                2..=5 => row.base[usize::from(field - 2)] += 1,
                6..=11 => row.affine[usize::from(field - 6)] += 1,
                12 => row.determinant += 1,
                13 => row.aabb.empty = !row.aabb.empty,
                14..=17 => row.aabb.edges[usize::from(field - 14)] += 1,
                _ => unreachable!(),
            }
        });
    }
    controls.check("geometry sentinel path", |log| {
        log.observations[0].projection.geometry[0].path = Some("root".to_owned());
    });
}

fn clips(controls: &mut Controls<'_>) {
    controls.check("clip order", |log| {
        log.observations[0].projection.clips.swap(0, 1);
    });
    controls.check("clip removed row", |log| {
        log.observations[0].projection.clips.pop();
    });
    controls.check("clip duplicate row", |log| {
        let row = log.observations[0].projection.clips[1].clone();
        log.observations[0].projection.clips.push(row);
    });
    for (label, field) in [
        ("clip key", 0_u8),
        ("clip owner", 1),
        ("clip path", 2),
        ("clip parent", 3),
        ("clip shape", 4),
        ("clip determinant", 5),
    ] {
        controls.check(label, move |log| {
            let row = &mut log.observations[0].projection.clips[1];
            match field {
                0 => row.key += 1,
                1 => row.owner += 1,
                2 => row.path.push('x'),
                3 => row.parent = None,
                4 => row.shape += 1,
                5 => row.determinant += 1,
                _ => unreachable!(),
            }
        });
    }
    affine_controls(controls, "clip affine", clip_affine);
    aabb_controls(controls, "clip primitive", clip_primitive);
    aabb_controls(controls, "clip effective", clip_effective);
    controls.check("clip none parent", |log| {
        log.observations[0].projection.clips[0].parent = Some(0);
    });
}

fn paints(controls: &mut Controls<'_>) {
    controls.check("paint order", |log| {
        log.observations[0].projection.paints.swap(0, 1);
    });
    controls.check("paint removed row", |log| {
        log.observations[0].projection.paints.pop();
    });
    controls.check("paint duplicate row", |log| {
        let row = log.observations[0].projection.paints[0].clone();
        log.observations[0].projection.paints.push(row);
    });
    for (label, field) in [
        ("paint key", 0_u8),
        ("paint owner", 1),
        ("paint path", 2),
        ("paint determinant", 3),
        ("paint coverage shape", 4),
        ("paint coverage brush", 5),
        ("paint clip", 6),
        ("paint stack", 7),
        ("paint item", 8),
    ] {
        controls.check(label, move |log| {
            let row = &mut log.observations[0].projection.paints[0];
            match field {
                0 => row.key += 1,
                1 => row.owner += 1,
                2 => row.path.push('x'),
                3 => row.determinant += 1,
                4 => {
                    let PaintReference::Coverage { shape, .. } = &mut row.reference else {
                        panic!("first paint should use coverage");
                    };
                    *shape += 1;
                }
                5 => {
                    let PaintReference::Coverage { brush, .. } = &mut row.reference else {
                        panic!("first paint should use coverage");
                    };
                    *brush += 1;
                }
                6 => row.clip = None,
                7 => row.stack += 1,
                8 => row.item += 1,
                _ => unreachable!(),
            }
        });
    }
    affine_controls(controls, "paint affine", paint_affine);
    aabb_controls(controls, "paint aabb", paint_aabb);
    controls.check("paint reference variant", |log| {
        log.observations[0].projection.paints[0].reference = PaintReference::Image { image: 0 };
    });
    controls.check("image reference", |log| {
        let PaintReference::Image { image } =
            &mut log.observations[0].projection.paints[2].reference
        else {
            panic!("third paint should use an image");
        };
        *image += 1;
    });
    controls.check("paint none clip", |log| {
        log.observations[0].projection.paints[1].clip = Some(0);
    });
}

fn items(controls: &mut Controls<'_>, semantic: bool) {
    let table = if semantic { "semantic" } else { "hit" };
    controls.check(&format!("{table} order"), move |log| {
        let rows = item_rows(log, semantic);
        rows.swap(0, 1);
    });
    controls.check(&format!("{table} removed row"), move |log| {
        item_rows(log, semantic).pop();
    });
    controls.check(&format!("{table} duplicate row"), move |log| {
        let row = item_rows(log, semantic)[0].clone();
        item_rows(log, semantic).push(row);
    });
    for (suffix, field) in [
        ("key", 0_u8),
        ("owner", 1),
        ("path", 2),
        ("determinant", 3),
        ("shape", 4),
        ("clip", 5),
        ("stack", 6),
        ("item", 7),
    ] {
        controls.check(&format!("{table} {suffix}"), move |log| {
            let row = &mut item_rows(log, semantic)[0];
            match field {
                0 => row.key += 1,
                1 => row.owner += 1,
                2 => row.path.push('x'),
                3 => row.determinant += 1,
                4 => row.shape += 1,
                5 => row.clip = None,
                6 => row.stack += 1,
                7 => row.item += 1,
                _ => unreachable!(),
            }
        });
    }
    let affine = if semantic {
        semantic_affine
    } else {
        hit_affine
    };
    let aabb = if semantic { semantic_aabb } else { hit_aabb };
    affine_controls(controls, &format!("{table} affine"), affine);
    aabb_controls(controls, &format!("{table} aabb"), aabb);
    controls.check(&format!("{table} none clip"), move |log| {
        item_rows(log, semantic)[1].clip = Some(0);
    });
}

fn item_rows(log: &mut Log, semantic: bool) -> &mut Vec<super::types::Item> {
    if semantic {
        &mut log.observations[0].projection.semantics
    } else {
        &mut log.observations[0].projection.hits
    }
}

type AffineSelector = for<'a> fn(&'a mut Log) -> &'a mut [i64; 6];
type AabbSelector = for<'a> fn(&'a mut Log) -> &'a mut Aabb;

fn affine_controls(controls: &mut Controls<'_>, label: &str, select: AffineSelector) {
    for component in 0..6 {
        controls.check(&format!("{label} {component}"), move |log| {
            select(log)[component] += 1;
        });
    }
}

fn aabb_controls(controls: &mut Controls<'_>, label: &str, select: AabbSelector) {
    controls.check(&format!("{label} empty"), move |log| {
        let value = select(log);
        value.empty = !value.empty;
    });
    for edge in 0..4 {
        controls.check(&format!("{label} edge {edge}"), move |log| {
            select(log).edges[edge] += 1;
        });
    }
}

fn clip_affine(log: &mut Log) -> &mut [i64; 6] {
    &mut log.observations[0].projection.clips[1].affine
}

fn clip_primitive(log: &mut Log) -> &mut Aabb {
    &mut log.observations[0].projection.clips[1].primitive
}

fn clip_effective(log: &mut Log) -> &mut Aabb {
    &mut log.observations[0].projection.clips[1].effective
}

fn paint_affine(log: &mut Log) -> &mut [i64; 6] {
    &mut log.observations[0].projection.paints[0].affine
}

fn paint_aabb(log: &mut Log) -> &mut Aabb {
    &mut log.observations[0].projection.paints[0].aabb
}

fn hit_affine(log: &mut Log) -> &mut [i64; 6] {
    &mut log.observations[0].projection.hits[0].affine
}

fn hit_aabb(log: &mut Log) -> &mut Aabb {
    &mut log.observations[0].projection.hits[0].aabb
}

fn semantic_affine(log: &mut Log) -> &mut [i64; 6] {
    &mut log.observations[0].projection.semantics[0].affine
}

fn semantic_aabb(log: &mut Log) -> &mut Aabb {
    &mut log.observations[0].projection.semantics[0].aabb
}
