use std::fmt;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{StationSpec, VisitStatus};

/// Station progress to reaching the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug)]
pub struct StationProgress {
    /// Progress bar to display this station's state and progress.
    pub progress_bar: ProgressBar,
    /// Whether this station has been visited.
    pub visit_status: VisitStatus,
}

impl StationProgress {
    /// Characters to use for the progress bar to have fine grained animation.
    pub const PROGRESS_CHARS: &'static str = "█▉▊▋▌▍▎▏  ";
    /// Template to apply when the station visit failed.
    pub const STYLE_FAILED: &'static str =
        "❌ {msg:20} [{bar:40.black.bright/red}] {bytes}/{total_bytes} ({elapsed:.yellow})";
    /// Template to apply when the station visit is in progress.
    pub const STYLE_IN_PROGRESS: &'static str = "{spinner:.green}{spinner:.green} {msg:20} [{bar:40.cyan/blue}] {pos}/{len} ({elapsed:.yellow} {eta})";
    /// Template to apply when the station visit is in progress.
    pub const STYLE_IN_PROGRESS_BYTES: &'static str = "{spinner:.green}{spinner:.green} {msg:20} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({elapsed:.yellow} {eta})";
    /// Template to apply when the station is not ready to be visited.
    pub const STYLE_NOT_READY: &'static str =
        "⏳ {msg:20} [{bar:40.blue.dim/blue}] {pos}/{len} (not ready)";
    /// Template to apply when a parent station has failed.
    pub const STYLE_PARENT_FAILED: &'static str =
        "☠️  {msg:20} [{bar:40.red/red.dim}] {pos}/{len} (parent failed)";
    /// Template to apply when the station is queued to be visited.
    pub const STYLE_QUEUED: &'static str =
        "⏳ {msg:20} [{bar:40.blue.dim/blue}] {pos}/{len} (queued)";
    /// Template to apply when the station visit is successful.
    pub const STYLE_SUCCESS: &'static str =
        "✅ {msg:20} [{bar:40.green/green}] {pos}/{len} ({elapsed:.yellow} Ok!)";
    /// Template to apply when the station visit is successful.
    pub const STYLE_SUCCESS_BYTES: &'static str =
        "✅ {msg:20} [{bar:40.green/green}] {bytes}/{total_bytes} ({elapsed:.yellow} Ok!)";
    /// Template to apply when the station was not necessary to visit.
    pub const STYLE_UNCHANGED_BYTES: &'static str = "✅ {msg:20} [{bar:40.green.dim/green}] {bytes}/{total_bytes} ({elapsed:.yellow} Unchanged)";

    /// Returns a new [`StationProgress`].
    ///
    /// # Parameters
    ///
    /// * `station_spec`: Behaviour specification for this station.
    /// * `visit_status`: Whether this [`StationProgress`] is ready to be
    ///   visited.
    pub fn new<E>(station_spec: &StationSpec<E>, visit_status: VisitStatus) -> Self {
        let id_style = Style::new().blue().bold();
        let name_style = Style::new().bold().bright();

        let message = format!(
            "{id} {name}",
            id = id_style.apply_to(station_spec.id()),
            name = name_style.apply_to(station_spec.name())
        );

        let progress_bar = ProgressBar::hidden();
        progress_bar.set_length(100);
        progress_bar.set_message(message);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(Self::STYLE_NOT_READY)
                .progress_chars(Self::PROGRESS_CHARS),
        );

        Self {
            progress_bar,
            visit_status,
        }
    }

    /// Sets the [`ProgressStyle`] for this station's [`ProgressBar`].
    pub fn with_progress_style(self, progress_style: ProgressStyle) -> Self {
        self.progress_bar.set_style(progress_style);
        self
    }

    /// Returns a type that implements [`fmt::Display`] for this progress.
    pub fn display<'f, E>(&'f self, station_spec: &'f StationSpec<E>) -> impl fmt::Display + 'f {
        StationProgressDisplay {
            station_spec,
            station_progress: self,
        }
    }
}

/// Implements `Display`
struct StationProgressDisplay<'station, E> {
    station_spec: &'station StationSpec<E>,
    station_progress: &'station StationProgress,
}

impl<E> fmt::Display for StationProgressDisplay<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] ", self.station_progress.visit_status)?;

        self.station_spec.fmt(f)
    }
}
