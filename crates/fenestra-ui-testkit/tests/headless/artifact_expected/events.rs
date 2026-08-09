const HEADLESS_ZERO: &str = concat!(
    "h-event|1|0|8001|0|build|none|observed|-|0|none|-|-|120|90|5|5|1|3|5|",
    "0|0|0|0|0|0|0|0|0|0"
);
const HEADLESS_STALE: &str = concat!(
    "h-event|1|19|8001|6|scheduler|pointer|failed:runtime|0|-|key:20|-|-|",
    "120|90|5|5|1|3|5|0|0|0|0|1|40|0|0|0|0"
);
const HEADLESS_FINAL: &str = concat!(
    "h-event|1|54|8001|19|scheduler|completion|stopped|-|-|none|-|-|90|70|",
    "5|5|0|2|4|0|0|0|0|0|0|0|0|0|0"
);

const SCHEDULER_ZERO: &str = concat!(
    "s-event|1|0|8001|1|callback-deferred|2|1|80|-|running|0|-|-|",
    "1|80|0|0|0|-|0|0|-|0|0|-|0|0|-|-|-|false"
);
const SCHEDULER_STALE: &str = concat!(
    "s-event|1|10|8001|6|action-transaction-missing-node|-|-|-|-|running|5|-|-|",
    "0|0|-|0|0|-|1|40|4|0|0|-|0|0|-|-|-|false"
);
const SCHEDULER_LOSS: &str = concat!(
    "s-event|1|34|8001|15|input-renderer-lost|0|accepted|-|-|running|9|-|1|",
    "0|0|-|1|32|0|1|40|1|1|40|4|1|96|4|0:1|0:0|false"
);
const SCHEDULER_DUPLICATE_SHUTDOWN: &str = concat!(
    "s-event|1|36|8001|15|input-shutdown|already-accepted|-|-|-|shutdown-queued|9|-|2|",
    "0|0|-|2|64|0|1|40|1|1|40|4|1|96|4|0:1|0:0|false"
);
const SCHEDULER_FINAL: &str = concat!(
    "s-event|1|40|8001|19|action-idle|-|-|-|-|stopped|9|-|-|",
    "0|0|-|0|0|-|0|0|-|0|0|-|0|0|-|0:1|0:1|false"
);

pub(super) fn assert_trace_sections(lines: &[&str]) {
    assert_eq!(lines.len(), 100);
    assert_eq!(lines[0], "headless-trace-begin|55|8800");
    assert_dense_events(&lines[1..56], "h-event", 30);
    assert_eq!(lines[1], HEADLESS_ZERO);
    assert_eq!(lines[20], HEADLESS_STALE);
    assert_eq!(lines[55], HEADLESS_FINAL);
    assert_eq!(lines[56], "headless-trace-end");

    assert_eq!(lines[57], "scheduler-trace-begin|41|3936");
    assert_dense_events(&lines[58..99], "s-event", 32);
    assert_eq!(lines[58], SCHEDULER_ZERO);
    assert_eq!(lines[68], SCHEDULER_STALE);
    assert_eq!(lines[92], SCHEDULER_LOSS);
    assert_eq!(lines[94], SCHEDULER_DUPLICATE_SHUTDOWN);
    assert_eq!(lines[98], SCHEDULER_FINAL);
    assert_eq!(lines[99], "scheduler-trace-end");
}

fn assert_dense_events(lines: &[&str], marker: &str, field_count: usize) {
    for (sequence, line) in lines.iter().enumerate() {
        let fields = line.split('|').collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            field_count,
            "wrong arity at {marker} {sequence}"
        );
        assert_eq!(fields[0], marker);
        assert_eq!(fields[1], "1");
        assert_eq!(fields[2], sequence.to_string());
        assert_eq!(fields[3], "8001");
    }
}
