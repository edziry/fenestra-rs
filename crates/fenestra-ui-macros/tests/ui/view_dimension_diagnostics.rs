use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 3;
        view responsive {
            rect panel {
                width: fill(0);
            }
        }
    };
    let _ = ui! {
        format 3;
        view responsive {
            rect panel {
                max_width: 40;
                min_width: 80;
            }
        }
    };
    let _ = ui! {
        format 3;
        view responsive {
            rect panel {
                height: fill(65536);
            }
        }
    };
}
