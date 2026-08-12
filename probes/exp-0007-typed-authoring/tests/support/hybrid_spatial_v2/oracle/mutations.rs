use super::compare::{assert_log_eq, compare_log};
use super::mutation_failure::failure_controls;
use super::mutation_projection::projection_controls;
use super::types::{Child, Hit, Log, ManifestEntry, Mutation, Value};

pub fn assert_mutation_controls(actual: &Log, expected: &Log) -> usize {
    assert_log_eq(actual, expected);
    let mut controls = Controls {
        actual,
        expected,
        count: 0,
    };
    top_level(&mut controls);
    failure_controls(&mut controls);
    state(&mut controls);
    receipts(&mut controls);
    projection_controls(&mut controls);
    queries_and_raster(&mut controls);
    controls.count
}

pub(super) struct Controls<'a> {
    actual: &'a Log,
    expected: &'a Log,
    count: usize,
}

impl Controls<'_> {
    pub(super) fn check(&mut self, label: &str, mutate: impl FnOnce(&mut Log)) {
        let mut changed = self.actual.clone();
        mutate(&mut changed);
        let difference = match compare_log(&changed, self.expected) {
            Ok(()) => panic!("comparator accepted mutation: {label}"),
            Err(difference) => difference,
        };
        assert!(
            !difference.to_string().is_empty(),
            "comparator returned an unlabeled difference: {label}"
        );
        self.count += 1;
    }
}

fn top_level(controls: &mut Controls<'_>) {
    controls.check("final key", |log| log.final_keys[0] ^= 1);
    controls.check("final key order", |log| log.final_keys.swap(0, 1));
    controls.check("empty no-op preservation", |log| {
        log.noop.empty_preserved = false;
    });
    controls.check("same-value no-op preservation", |log| {
        log.noop.same_value_preserved = false;
    });
    controls.check("round-trip no-op preservation", |log| {
        log.noop.round_trip_preserved = false;
    });
    controls.check("observation order", |log| log.observations.swap(0, 1));
    controls.check("generation", |log| log.observations[0].generation += 1);
    controls.check("viewport width", |log| log.observations[0].viewport[0] += 1);
    controls.check("viewport height", |log| {
        log.observations[0].viewport[1] += 1
    });
}

fn state(controls: &mut Controls<'_>) {
    controls.check("node order", |log| {
        log.observations[0].state.nodes.swap(0, 1)
    });
    controls.check("node path", |log| {
        log.observations[0].state.nodes[0].path.push('x');
    });
    controls.check("node parent", |log| {
        log.observations[0].state.nodes[1].parent = None;
    });
    controls.check("root none parent", |log| {
        log.observations[0].state.nodes[0].parent = Some("root".to_owned());
    });
    controls.check("node template", |log| {
        log.observations[0].state.nodes[0].template += 1;
    });
    controls.check("node component", |log| {
        log.observations[0].state.nodes[0].component += 1;
    });
    controls.check("property order", |log| {
        log.observations[0].state.nodes[0].properties.swap(0, 1);
    });
    controls.check("property id", |log| {
        log.observations[0].state.nodes[0].properties[0].0 += 1;
    });
    for (label, property) in [
        ("i32 property", 0),
        ("bool property", 6),
        ("policy property", 7),
    ] {
        controls.check(label, move |log| {
            mutate_value(&mut log.observations[0].state.nodes[0].properties[property].1);
        });
    }
    for channel in 0..4 {
        controls.check(&format!("rgba property channel {channel}"), move |log| {
            let Value::Rgba(value) = &mut log.observations[0].state.nodes[0].properties[4].1 else {
                panic!("property four should be RGBA");
            };
            value[channel] ^= 1;
        });
    }
    controls.check("property value variant", |log| {
        log.observations[0].state.nodes[0].properties[0].1 = Value::Bool(false);
    });
    controls.check("child order", |log| {
        log.observations[0].state.nodes[0].children.swap(0, 1);
    });
    controls.check("static child", |log| {
        let Child::Static(path) = &mut log.observations[0].state.nodes[0].children[0] else {
            panic!("fixture child should be static");
        };
        path.push('x');
    });
    controls.check("region child", |log| {
        let Child::Region(path) = &mut log.observations[0].state.nodes[2].children[1] else {
            panic!("fixture child should be a region");
        };
        path.push('x');
    });
    controls.check("child variant", |log| {
        let Child::Static(path) = &log.observations[0].state.nodes[0].children[0] else {
            panic!("fixture child should be static");
        };
        log.observations[0].state.nodes[0].children[0] = Child::Region(path.clone());
    });
    controls.check("fragment path", |log| {
        log.observations[0].state.fragments[0].path.push('x');
    });
    controls.check("fragment descriptor", |log| {
        log.observations[0].state.fragments[0].descriptor += 1;
    });
    controls.check("fragment member order", |log| {
        log.observations[0].state.fragments[0].members.swap(0, 1);
    });
    controls.check("fragment member key", |log| {
        log.observations[0].state.fragments[0].members[0].0 += 1;
    });
    controls.check("fragment member path", |log| {
        log.observations[0].state.fragments[0].members[0]
            .1
            .push('x');
    });
}

fn receipts(controls: &mut Controls<'_>) {
    controls.check("receipt generation", |log| {
        log.observations[1].receipt.generation += 1;
    });
    controls.check("receipt invalidation", |log| {
        log.observations[1].receipt.invalidation.pop();
    });
    controls.check("receipt invalidation order", |log| {
        log.observations[1].receipt.invalidation.swap(0, 1);
    });
    controls.check("receipt invalidation class", |log| {
        log.observations[1].receipt.invalidation[0] ^= 1;
    });
    controls.check("receipt duplicate mutation", |log| {
        let mutation = log.observations[1].receipt.mutations[0].clone();
        log.observations[1].receipt.mutations.push(mutation);
    });
    controls.check("receipt mutation variant", |log| {
        log.observations[1].receipt.mutations[0] = Mutation::Move {
            fragment: "root/r:0".to_owned(),
            key: 0,
            root: "root".to_owned(),
            old_index: 0,
            final_index: 0,
        };
    });
    controls.check("viewport old", |log| {
        let Mutation::Viewport { old, .. } = &mut log.observations[1].receipt.mutations[0] else {
            panic!("step one should resize");
        };
        old[0] += 1;
    });
    controls.check("viewport new", |log| {
        let Mutation::Viewport { new, .. } = &mut log.observations[1].receipt.mutations[0] else {
            panic!("step one should resize");
        };
        new[1] += 1;
    });
    property_mutation_controls(controls);
    insert_mutation_controls(controls);
    move_mutation_controls(controls);
    remove_mutation_controls(controls);
}

fn property_mutation_controls(controls: &mut Controls<'_>) {
    controls.check("property mutation node", |log| {
        property_mutation(log).0.push('x')
    });
    controls.check("property mutation id", |log| *property_mutation(log).1 += 1);
    controls.check("property mutation old", |log| {
        mutate_value(property_mutation(log).2)
    });
    controls.check("property mutation new", |log| {
        mutate_value(property_mutation(log).3)
    });
}

fn property_mutation(log: &mut Log) -> (&mut String, &mut u32, &mut Value, &mut Value) {
    let Mutation::Property {
        node,
        property,
        old,
        new,
    } = &mut log.observations[2].receipt.mutations[0]
    else {
        panic!("step two should change a property");
    };
    (node, property, old, new)
}

fn insert_mutation_controls(controls: &mut Controls<'_>) {
    for (label, mutate) in [
        ("insert fragment", 0_u8),
        ("insert key", 1),
        ("insert root", 2),
        ("insert index", 3),
        ("insert manifest", 4),
        ("insert manifest variant", 5),
    ] {
        controls.check(label, move |log| {
            let Mutation::Insert {
                fragment,
                key,
                root,
                final_index,
                created,
            } = &mut log.observations[5].receipt.mutations[0]
            else {
                panic!("step five should insert");
            };
            match mutate {
                0 => fragment.push('x'),
                1 => *key += 1,
                2 => root.push('x'),
                3 => *final_index += 1,
                4 => manifest_path(&mut created[0]).push('x'),
                5 => {
                    let path = manifest_path(&mut created[0]).clone();
                    created[0] = ManifestEntry::Fragment(path);
                }
                _ => unreachable!(),
            }
        });
    }
}

fn move_mutation_controls(controls: &mut Controls<'_>) {
    for (label, field) in [
        ("move fragment", 0_u8),
        ("move key", 1),
        ("move root", 2),
        ("move old index", 3),
        ("move final index", 4),
    ] {
        controls.check(label, move |log| {
            let Mutation::Move {
                fragment,
                key,
                root,
                old_index,
                final_index,
            } = &mut log.observations[6].receipt.mutations[0]
            else {
                panic!("step six should move");
            };
            match field {
                0 => fragment.push('x'),
                1 => *key += 1,
                2 => root.push('x'),
                3 => *old_index += 1,
                4 => *final_index += 1,
                _ => unreachable!(),
            }
        });
    }
}

fn remove_mutation_controls(controls: &mut Controls<'_>) {
    for (label, field) in [
        ("remove fragment", 0_u8),
        ("remove key", 1),
        ("remove root", 2),
        ("remove old index", 3),
        ("remove manifest", 4),
        ("remove manifest variant", 5),
    ] {
        controls.check(label, move |log| {
            let Mutation::Remove {
                fragment,
                key,
                root,
                old_index,
                retired,
            } = &mut log.observations[8].receipt.mutations[0]
            else {
                panic!("step eight should remove");
            };
            match field {
                0 => fragment.push('x'),
                1 => *key += 1,
                2 => root.push('x'),
                3 => *old_index += 1,
                4 => manifest_path(&mut retired[0]).push('x'),
                5 => {
                    let path = manifest_path(&mut retired[0]).clone();
                    retired[0] = ManifestEntry::Fragment(path);
                }
                _ => unreachable!(),
            }
        });
    }
}

fn queries_and_raster(controls: &mut Controls<'_>) {
    let hit_index = controls.actual.observations[0]
        .hit_queries
        .iter()
        .position(|query| query.result.is_some())
        .expect("the fixture should contain a hit");
    controls.check("hit query order", |log| {
        log.observations[0].hit_queries.swap(0, 1);
    });
    controls.check("hit query scene x", |log| {
        log.observations[0].hit_queries[0].scene[0] += 1
    });
    controls.check("hit query scene y", |log| {
        log.observations[0].hit_queries[0].scene[1] += 1
    });
    controls.check("hit answer", move |log| {
        log.observations[0].hit_queries[hit_index].result = None;
    });
    for (label, field) in [
        ("hit key", 0_u8),
        ("hit owner", 1),
        ("hit path", 2),
        ("hit item", 3),
        ("hit local x", 4),
        ("hit local y", 5),
    ] {
        controls.check(label, move |log| {
            let hit = log.observations[0].hit_queries[hit_index]
                .result
                .as_mut()
                .expect("selected query should hit");
            match field {
                0 => hit.key += 1,
                1 => hit.owner += 1,
                2 => hit.path.push('x'),
                3 => hit.item += 1,
                4 => hit.local[0] += 1,
                5 => hit.local[1] += 1,
                _ => unreachable!(),
            }
        });
    }
    controls.check("none hit variant", |log| {
        log.observations[0].hit_queries[0].result = Some(Hit {
            key: 0,
            owner: 1,
            path: "root".to_owned(),
            item: 0,
            local: [0, 0],
        });
    });
    controls.check("raster width", |log| log.observations[0].raster.width += 1);
    controls.check("raster height", |log| {
        log.observations[0].raster.height += 1
    });
    controls.check("raster stride", |log| {
        log.observations[0].raster.stride += 1
    });
    controls.check("raster byte", |log| {
        log.observations[0].raster.bytes[0] ^= 1
    });
}

fn manifest_path(entry: &mut ManifestEntry) -> &mut String {
    match entry {
        ManifestEntry::Node(path) | ManifestEntry::Fragment(path) => path,
    }
}

fn mutate_value(value: &mut Value) {
    match value {
        Value::Bool(value) | Value::Policy(value) => *value = !*value,
        Value::I32(value) => *value += 1,
        Value::Rgba(value) => value[0] ^= 1,
    }
}
