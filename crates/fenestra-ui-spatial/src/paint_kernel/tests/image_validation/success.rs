use super::*;

#[test]
fn valid_image_proof_retains_exact_metadata_bytes_and_commits_pixels_once() {
    let bytes = vec![
        0, 0, 0, 0, // transparent
        64, 32, 0, 128, // nontrivial premultiplied
        255, 254, 253, 255, // opaque
        128, 128, 128, 128, // equality is valid
        1, 2, 3, 4, // ascending channels
        9, 8, 7, 10, // distinct final pixel
    ];
    let image = image(3, 2, 12, bytes.clone());
    let mut accepted = 10;
    let proof = match prepare_image_p4(
        &image,
        &mut accepted,
        IMAGE_EDGE_MAXIMUM,
        IMAGE_PIXELS_MAXIMUM,
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("expected valid P4 image proof"),
    };

    assert_eq!(proof.width(), 3);
    assert_eq!(proof.height(), 2);
    assert_eq!(proof.stride(), 12);
    assert_eq!(proof.bytes(), bytes.as_slice());
    assert_eq!(image.bytes(), bytes.as_slice());
    assert_eq!(accepted, 16);
}
