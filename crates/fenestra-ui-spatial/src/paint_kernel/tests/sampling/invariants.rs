use super::*;

#[test]
fn every_quantized_parameter_remains_premultiplied_and_the_proof_is_reusable() {
    let stops = [
        stop(0, color(255, 20, 0, 10)),
        stop(7_000, color(0, 255, 200, 64)),
        stop(31_000, color(100, 30, 250, 128)),
        stop(31_000, color(255, 10, 20, 200)),
        stop(u16::MAX, color(255, 255, 255, 255)),
    ];
    let start = point(0, 0);
    let end = point(i64::from(u16::MAX), 0);
    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        stops.len() as u32,
        start,
        end,
        &stops,
        stops.len(),
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("invariant fixture must prepare"),
    };
    let offsets: Vec<_> = (0..proof.stop_count())
        .map(|index| proof.stop(index).offset())
        .collect();
    let colors: Vec<_> = (0..proof.stop_count())
        .map(|index| proof.stop(index).color())
        .collect();

    for parameter in 0..=u16::MAX {
        let sampled = sample_gradient_p3(&proof, point(i64::from(parameter), 0));
        assert!(sampled.r() <= sampled.a());
        assert!(sampled.g() <= sampled.a());
        assert!(sampled.b() <= sampled.a());
    }

    assert_eq!(proof.start(), start);
    assert_eq!(proof.end(), end);
    assert_eq!(proof.stop_count(), stops.len());
    for (index, (&offset, &color)) in offsets.iter().zip(&colors).enumerate() {
        assert_eq!(proof.stop(index).offset(), offset);
        assert_eq!(proof.stop(index).color(), color);
    }
}
