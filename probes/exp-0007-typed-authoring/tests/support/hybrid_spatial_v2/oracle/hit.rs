use super::coverage::{coverage_contains, shape_contains};
use super::numeric::{Point, SCALE, contains, inverse_point};
use super::projection::shape_bounds;
use super::scene::Scene;
use super::types::{Hit, HitQuery, Projection};

pub fn queries(scene: &Scene, projection: &Projection, viewport: [i32; 2]) -> Vec<HitQuery> {
    let width = u32::try_from(viewport[0]).expect("viewport width should be positive");
    let height = u32::try_from(viewport[1]).expect("viewport height should be positive");
    let mut queries =
        Vec::with_capacity(usize::try_from(width * height).expect("pixel count should fit") + 4);
    for y in 0..height {
        for x in 0..width {
            queries.push(query(
                scene,
                projection,
                [
                    i64::from(x) * SCALE + SCALE / 2,
                    i64::from(y) * SCALE + SCALE / 2,
                ],
            ));
        }
    }
    for point in [
        [-1, 0],
        [0, -1],
        [i64::from(width) * SCALE, 0],
        [0, i64::from(height) * SCALE],
    ] {
        queries.push(query(scene, projection, point));
    }
    queries
}

pub fn query(scene: &Scene, projection: &Projection, point: Point) -> HitQuery {
    let result = scene
        .hits
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, hit)| {
            if !hit.accepts {
                return None;
            }
            if let Some(clip) = hit.clip
                && !clip_contains(scene, projection, clip, point)
            {
                return None;
            }
            let row = &projection.hits[index];
            if !contains(row.aabb, point) {
                return None;
            }
            let local = inverse_point(row.affine, point);
            if !coverage_contains(scene, hit.coverage, hit.bounds, local) {
                return None;
            }
            Some(Hit {
                key: row.key,
                owner: hit.owner,
                path: row.path.clone(),
                item: hit.item,
                local,
            })
        });
    HitQuery {
        scene: point,
        result,
    }
}

pub fn clip_contains(scene: &Scene, projection: &Projection, terminal: u32, point: Point) -> bool {
    let terminal_index = usize::try_from(terminal).expect("clip should fit");
    if !contains(projection.clips[terminal_index].effective, point) {
        return false;
    }
    let mut current = Some(terminal);
    while let Some(key) = current {
        let index = usize::try_from(key).expect("clip should fit");
        let clip = &scene.clips[index];
        let row = &projection.clips[index];
        if !contains(row.primitive, point) {
            return false;
        }
        let local = inverse_point(row.affine, point);
        if !shape_contains(
            scene,
            clip.shape,
            clip.rule,
            shape_bounds(scene, clip.shape),
            local,
        ) {
            return false;
        }
        current = clip.parent;
    }
    true
}
