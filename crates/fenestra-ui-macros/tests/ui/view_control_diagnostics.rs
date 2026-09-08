use fenestra_ui_macros::ui;

fn main() {
    let _ = ui! {
        format 3;
        view example {
            button save {
                label: " \t\n";
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            checkbox check {
                label: "Check";
                checked: 1;
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            button outer {
                label: "Outer";
                disabled: true;
                row wrapper {
                    checkbox inner {
                        label: "Inner";
                        disabled: true;
                    }
                }
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            checkbox check {
                rect child {
                    input: accept;
                }
                label: "Check";
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            button save {
                label: "Save";
                rect child {
                    focus_color: rgba8(1,2,3,4);
                }
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            button save {
                label: r#"Save // /* a label */"#;
                text caption {
                    content: "Save \u{4e16}";
                    checked_color: rgba8(1,2,3,4);
                }
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            button save {
                label: "Save";
                disabled_color: rgba8(1,2,3,4);
            }
        }
    };
    let _ = ui! {
        format 3;
        view example {
            column outside {
                button save {
                    label: "Save";
                }
                hover_background: rgba8(1,2,3,4);
            }
        }
    };
}
