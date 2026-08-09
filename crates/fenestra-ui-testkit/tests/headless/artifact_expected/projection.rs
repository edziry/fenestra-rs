const EXPECTED: [&str; 30] = [
    "projection-begin|9|90|70|5|5|0|2|4",
    "computed-begin",
    "computed|root|84|80|rgba8:090909ff|true|ignore",
    "computed|root/s:0|80|50|rgba8:020202ff|true|ignore",
    "computed|root/s:0/s:0|30|10|rgba8:141e28ff|false|accept",
    "computed|root/s:0/m:1:10|40|12|rgba8:505a64ff|true|accept",
    "computed|root/s:0/m:1:30|40|14|rgba8:505a64ff|true|accept",
    "computed-end",
    "geometry-begin",
    "geometry|root|0|0|84|80|0|0|84|70",
    "geometry|root/s:0|0|0|80|50|0|0|80|50",
    "geometry|root/s:0/s:0|0|0|30|10|0|0|30|10",
    "geometry|root/s:0/m:1:10|0|10|40|12|0|10|40|12",
    "geometry|root/s:0/m:1:30|0|22|40|14|0|22|40|14",
    "geometry-end",
    "semantic-begin",
    "semantic-end",
    "hit-begin",
    "hit|root/s:0/m:1:10|0|10|40|12",
    "hit|root/s:0/m:1:30|0|22|40|14",
    "hit-end",
    "scene-begin",
    "scene|root|0|0|84|70|rgba8:090909ff",
    "scene|root/s:0|0|0|80|50|rgba8:020202ff",
    "scene|root/s:0/m:1:10|0|10|40|12|rgba8:505a64ff",
    "scene|root/s:0/m:1:30|0|22|40|14|rgba8:505a64ff",
    "scene-end",
    "projection-end",
    "result|pass",
    "end",
];

pub(super) fn assert_projection_and_result(lines: &[&str]) {
    assert_eq!(lines, EXPECTED);
    let record_count = lines
        .iter()
        .filter(|line| {
            matches!(
                line.split('|').next(),
                Some("computed" | "geometry" | "semantic" | "hit" | "scene")
            )
        })
        .count();
    assert_eq!(record_count, 16);
}
