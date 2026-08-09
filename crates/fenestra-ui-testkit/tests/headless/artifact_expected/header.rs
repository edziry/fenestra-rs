const EXPECTED: [&str; 14] = [
    "fenestra-headless-spine|1",
    "versions|fixture|1|schema|1|construction|1|style|1|trace|1|projection|1",
    "fixture|headless-spine|1|1|8001|1|1|1",
    "environment|platform|headless-fake|clock|scheduler|domain|8001",
    "projection-choices|full|vertical|rebuilt|reverse",
    "capacity-ir|1|5|4|1|3|12|2|3|5",
    "capacity-style|2",
    "capacity-runtime|8|8|8|2|40|3",
    "capacity-projection|8|8|1|8|8",
    "capacity-scheduler|1|80|8|4|128|8|1|40|8|2|80|8",
    "capacity-renderer|2|192|8",
    "capacity-scheduler-trace|256|24576|96",
    "capacity-headless-trace|128|20480|160",
    "capacity-artifact|65536|1024|512",
];

pub(super) fn assert_header(lines: &[&str]) {
    assert_eq!(lines, EXPECTED);
}
