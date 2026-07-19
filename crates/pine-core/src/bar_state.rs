//! Where a bar sits in the dataset, and where it is in its lifecycle.

/// The state of a bar, as the data source sees it.
///
/// Scripts read this through the `barstate.*` variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BarState {
    /// The first bar of the dataset.
    First,
    /// A closed historical bar.
    #[default]
    History,
    /// The last historical bar: the next one is realtime.
    LastConfirmedHistory,
    /// A realtime bar that is still receiving updates.
    Realtime,
    /// The last bar of the dataset.
    Last,
}

impl BarState {
    /// The bar is historical rather than live.
    pub fn is_history(self) -> bool {
        !matches!(self, BarState::Realtime)
    }

    /// The bar is still being updated by a live market.
    pub fn is_realtime(self) -> bool {
        matches!(self, BarState::Realtime)
    }

    /// This is the final calculation for the bar. Only a realtime bar can be
    /// recalculated, so every other state is already closed.
    pub fn is_confirmed(self) -> bool {
        !self.is_realtime()
    }

    /// The last bar in the set. Realtime bars are always last.
    pub fn is_last(self) -> bool {
        matches!(self, BarState::Last | BarState::Realtime)
    }

    pub fn is_first(self) -> bool {
        matches!(self, BarState::First)
    }

    pub fn is_last_confirmed_history(self) -> bool {
        matches!(self, BarState::LastConfirmedHistory)
    }
}
