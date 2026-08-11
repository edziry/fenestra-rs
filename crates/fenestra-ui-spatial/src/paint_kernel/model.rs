use crate::brush::SpatialRgba8V2;
use crate::model::SpatialPointV2;

pub(super) struct PreparedGradientStopP2 {
    offset: u16,
    color: SpatialRgba8V2,
}

impl PreparedGradientStopP2 {
    pub(super) const fn new(offset: u16, color: SpatialRgba8V2) -> Self {
        Self { offset, color }
    }

    pub(super) const fn offset(&self) -> u16 {
        self.offset
    }

    pub(super) const fn color(&self) -> SpatialRgba8V2 {
        self.color
    }
}

pub(crate) struct PreparedGradientP2 {
    start: SpatialPointV2,
    end: SpatialPointV2,
    stops: Vec<PreparedGradientStopP2>,
}

impl PreparedGradientP2 {
    pub(super) fn new(
        start: SpatialPointV2,
        end: SpatialPointV2,
        stops: Vec<PreparedGradientStopP2>,
    ) -> Self {
        Self { start, end, stops }
    }

    pub(super) const fn start(&self) -> SpatialPointV2 {
        self.start
    }

    pub(super) const fn end(&self) -> SpatialPointV2 {
        self.end
    }

    pub(super) fn stop_count(&self) -> usize {
        self.stops.len()
    }

    pub(super) fn stop(&self, index: usize) -> &PreparedGradientStopP2 {
        &self.stops[index]
    }

    #[cfg(test)]
    pub(crate) fn facts(&self) -> (SpatialPointV2, SpatialPointV2, Vec<(u16, SpatialRgba8V2)>) {
        (
            self.start,
            self.end,
            self.stops
                .iter()
                .map(|stop| (stop.offset, stop.color))
                .collect(),
        )
    }
}
