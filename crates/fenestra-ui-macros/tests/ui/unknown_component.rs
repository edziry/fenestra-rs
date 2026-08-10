use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 1;
        schema namespace 1 revision 1 {
            component panel = 0 {
                property width = 0: scalar_i32 = 0 invalidates [layout];
            }
        }
        construction {
            template root = 0: secret_missing_component {
                child region rows;
            }
            region rows = 0 owner root repeat root keys [] invalidates [structure];
        }
        style {}
    };
}
