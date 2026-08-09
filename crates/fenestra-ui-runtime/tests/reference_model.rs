mod support;

use std::collections::HashMap;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{NodeId, UiRuntime};

use support::model::CleanModel;
use support::{KEY, LIST, SECOND_KEY, VALUE, WIDTH, capacity, construction};

fn identities(runtime: &UiRuntime) -> HashMap<u64, NodeId> {
    let committed = runtime.committed();
    let list = committed.fragment(committed.root(), LIST).unwrap();
    committed.keyed_members(list).unwrap().collect()
}

#[test]
fn deterministic_steps_match_clean_reconstruction_after_every_commit() {
    let construction = construction();
    let mut model = CleanModel::from_construction(&construction);
    let mut runtime = UiRuntime::new(construction, capacity()).unwrap();
    model.initial_keys_are_present();
    model.assert_matches(&runtime.committed());
    let initial = identities(&runtime);

    let root = runtime.committed().root();
    let mut direct = runtime.begin_transaction();
    direct
        .set_property(root, WIDTH, PropertyValue::ScalarI32(130))
        .unwrap();
    drop(runtime.commit(direct).unwrap());
    model.set_root_width(130);
    model.assert_matches(&runtime.committed());
    assert_eq!(identities(&runtime), initial);

    let list = runtime.committed().fragment(root, LIST).unwrap();
    let mut insert = runtime.begin_transaction();
    insert.insert_keyed(list, 9, 1).unwrap();
    drop(runtime.commit(insert).unwrap());
    model.insert(9, 1);
    model.assert_matches(&runtime.committed());
    let after_insert = identities(&runtime);
    assert_eq!(after_insert[&KEY], initial[&KEY]);
    assert_eq!(after_insert[&SECOND_KEY], initial[&SECOND_KEY]);

    let mut update = runtime.begin_transaction();
    update
        .update_keyed(list, 9, VALUE, PropertyValue::ScalarI32(90))
        .unwrap();
    drop(runtime.commit(update).unwrap());
    model.update(9, 90);
    model.assert_matches(&runtime.committed());
    assert_eq!(identities(&runtime), after_insert);

    let mut movement = runtime.begin_transaction();
    movement.move_keyed(list, SECOND_KEY, 0).unwrap();
    drop(runtime.commit(movement).unwrap());
    model.move_key(SECOND_KEY, 0);
    model.assert_matches(&runtime.committed());
    assert_eq!(identities(&runtime), after_insert);

    let retired = identities(&runtime)[&KEY];
    let mut removal = runtime.begin_transaction();
    removal.remove_keyed(list, KEY).unwrap();
    drop(runtime.commit(removal).unwrap());
    model.remove(KEY);
    model.assert_matches(&runtime.committed());
    assert!(!identities(&runtime).values().any(|node| *node == retired));

    let mut reinsert = runtime.begin_transaction();
    reinsert.insert_keyed(list, KEY, 2).unwrap();
    drop(runtime.commit(reinsert).unwrap());
    model.insert(KEY, 2);
    model.assert_matches(&runtime.committed());
    assert_ne!(identities(&runtime)[&KEY], retired);
}
