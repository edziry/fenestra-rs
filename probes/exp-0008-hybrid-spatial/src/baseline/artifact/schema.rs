pub(super) const HEADER: &str = "spatial-v2|artifact=2|contract=2|corpus=2|kind=baseline";
pub(super) const PACKAGES: &str =
    "packages|probe=0.2.0|ir=0.2.0|layout=0.2.0|spatial=0.2.0|runtime=0.2.0";
pub(super) const PROFILE: &str =
    "profile|spatial=registered-v2|raster=registered-v2|candidate-count=0";
pub(super) const LIMITS: &str = concat!(
    "limits|spatial=256,1024,256,512,1024,512,256,256,4096,4096,2048,64,32,",
    "64,64,128,192,256,1024,256,32,4096,4194304,32,64,64,4096,65536,192,256|",
    "raster-pixels=4194304|records=4096|line-bytes=1024|artifact-bytes=1048576"
);

pub(super) const CASE_NAMES: [&str; 14] = [
    "all-layout",
    "all-free",
    "free-to-layout",
    "layout-to-free",
    "mixed-siblings",
    "transparent-wrapper",
    "split-geometry",
    "transformed-clip",
    "polygon-path",
    "rich-paint",
    "anchor-forward",
    "zero-extent",
    "runtime-mutation",
    "runtime-rollback",
];

pub(super) const OBSERVATION_COUNTS: [usize; 14] = [2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 9, 1];

pub(super) const CONTROL_FAMILIES: [&str; 7] = [
    "metadata", "records", "fields", "queries", "raster", "faults", "codec",
];

pub(super) const SPATIAL_LIMITS: [usize; 30] = [
    256, 1024, 256, 512, 1024, 512, 256, 256, 4096, 4096, 2048, 64, 32, 64, 64, 128, 192, 256,
    1024, 256, 32, 4096, 4_194_304, 32, 64, 64, 4096, 65_536, 192, 256,
];
