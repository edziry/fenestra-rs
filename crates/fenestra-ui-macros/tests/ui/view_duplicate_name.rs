use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 3;
        view hello {
            row root {
                rect card {}
                rect card {}
            }
        }
    };
}
