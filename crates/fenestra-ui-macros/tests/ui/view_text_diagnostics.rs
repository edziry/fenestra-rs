use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 3;
        view hello {
            text title {
                content: "caf\u{e9} // /* text */";
                font_size: 513;
            }
        }
    };
    let _ = ui! {
        format 3;
        view hello {
            text title {
                content: "hello";
                width: "12";
            }
        }
    };
    let _ = ui! {
        format 3;
        view hello {
            text title {
            }
        }
    };
}
