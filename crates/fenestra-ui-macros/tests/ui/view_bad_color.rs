use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 3;
        view hello {
            rect root {
                background: rgba8(0, 256, 0, 255);
            }
        }
    };
}
