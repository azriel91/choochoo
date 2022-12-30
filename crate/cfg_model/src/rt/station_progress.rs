use std::fmt;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    rt::{OpStatus, ProgressLimit},
    StationSpec,
};

/// Station progress to reaching the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug)]
pub struct StationProgress {
    /// Whether this station has been visited.
    pub op_status: OpStatus,
    /// Progress bar to display this station's state and progress.
    progress_bar: ProgressBar,
    /// Unit of measurement and limit to indicate progress.
    progress_limit: ProgressLimit,
}

impl StationProgress {
    /// Characters to use for the progress bar to have fine grained animation.
    pub const PROGRESS_CHARS: &'static str = "█▉▊▋▌▍▎▏  ";

    /// Returns a new [`StationProgress`].
    ///
    /// # Parameters
    ///
    /// * `station_spec`: Behaviour specification of the station.
    /// * `progress_limit`: Unit of measurement and limit to indicate progress.
    pub fn new<E>(station_spec: &StationSpec<E>, progress_limit: ProgressLimit) -> Self
    where
        E: 'static,
    {
        let op_status = OpStatus::SetupQueued;
        let progress_bar = ProgressBar::hidden();

        let message = {
            let id_style = Style::new().blue().bold();
            let name_style = Style::new().bold().bright();

            format!(
                "{id} {name}",
                id = id_style.apply_to(station_spec.id()),
                name = name_style.apply_to(station_spec.name())
            )
        };
        progress_bar.set_message(message);

        let station_progress = Self {
            op_status,
            progress_bar,
            progress_limit,
        };

        station_progress.progress_style_update();

        station_progress
    }

    /// Returns a reference to the [`ProgressBar`].
    pub fn progress_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }

    /// Steps the progress by 1.
    pub fn tick(&mut self) {
        self.progress_bar.tick();
    }

    /// Returns a type that implements [`fmt::Display`] for this progress.
    pub fn display<'f, E>(&'f self, station_spec: &'f StationSpec<E>) -> impl fmt::Display + 'f {
        StationProgressDisplay {
            station_spec,
            station_progress: self,
        }
    }

    /// Updates the progress limit.
    pub fn progress_limit_set(&mut self, progress_limit: ProgressLimit) {
        self.progress_limit = progress_limit;
        self.progress_style_update();
    }

    /// Updates the style of the progress bar.
    pub fn progress_style_update(&self) {
        let progress_length = match self.progress_limit {
            ProgressLimit::Unknown => 0, // indicatif uses `0` for spinner type progress bars.
            ProgressLimit::Steps(n) | ProgressLimit::Bytes(n) => n,
        };

        let progress_style_template =
            Self::progress_style_template(self.op_status, self.progress_limit);
        self.progress_bar.set_length(progress_length);

        self.progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(progress_style_template.as_str())
                .progress_chars(StationProgress::PROGRESS_CHARS),
        );

        // Finish the progress bar if our progress is complete.
        match self.op_status {
            OpStatus::SetupQueued
            | OpStatus::SetupSuccess
            | OpStatus::ParentPending
            | OpStatus::OpQueued
            | OpStatus::WorkInProgress => {}
            OpStatus::SetupFail
            | OpStatus::ParentFail
            | OpStatus::CheckFail
            | OpStatus::WorkFail => {
                self.progress_bar.abandon();
            }
            OpStatus::WorkSuccess | OpStatus::WorkUnnecessary => {
                self.progress_bar.finish();
            }
        }

        // Redraw the progress bar
        self.progress_bar.tick();
    }

    fn progress_style_template(op_status: OpStatus, progress_limit: ProgressLimit) -> String {
        let (symbol, status) = match op_status {
            OpStatus::SetupQueued => ("⏳", "setup queued"),
            OpStatus::SetupSuccess => ("⏳", "setup success"),
            OpStatus::SetupFail => ("❌", "setup fail"),
            OpStatus::ParentPending => ("⏰", "parent pending"),
            OpStatus::ParentFail => ("☠️ ", "parent fail"), // Extra space is deliberate
            OpStatus::OpQueued => ("⏳", "visit queued"),
            OpStatus::CheckFail => ("❌", "check fail"),
            OpStatus::WorkInProgress => ("{spinner:.green}{spinner:.green}", "in progress"),
            OpStatus::WorkUnnecessary => ("✅", "visit unnecessary"),
            OpStatus::WorkSuccess => ("✅", "visit success"),
            OpStatus::WorkFail => ("❌", "visit fail"),
        };

        let progress_bar = match op_status {
            OpStatus::SetupQueued => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .black()
                .dim(),
            OpStatus::SetupSuccess => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .blue()
                .dim(),
            OpStatus::SetupFail => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .magenta()
                .dim(),
            OpStatus::ParentPending => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .blue()
                .dim(),
            OpStatus::ParentFail => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .black()
                .dim(),
            OpStatus::OpQueued => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒")
                .blue()
                .dim(),
            OpStatus::CheckFail => console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒").red(),
            OpStatus::WorkInProgress => console::style("{bar:40.green.on_17}"),
            OpStatus::WorkUnnecessary => console::style("{bar:40.green.dim}"),
            OpStatus::WorkSuccess => console::style("{bar:40.green}"),
            OpStatus::WorkFail => console::style("{bar:40.red.dim}"),
        };

        let units = match progress_limit {
            ProgressLimit::Unknown => "",
            ProgressLimit::Steps(_) => "{pos}/{len}",
            ProgressLimit::Bytes(_) => "{bytes}/{total_bytes}",
        };

        format!("{symbol} {{msg:20}} [{progress_bar}] {units} ({status})")
    }
}

/// Implements `Display`
struct StationProgressDisplay<'station, E> {
    station_spec: &'station StationSpec<E>,
    station_progress: &'station StationProgress,
}

impl<E> fmt::Display for StationProgressDisplay<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] ", self.station_progress.op_status)?;

        self.station_spec.fmt(f)
    }
}
