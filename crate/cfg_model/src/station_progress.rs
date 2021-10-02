use std::fmt;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{ProgressLimit, StationSpec, VisitStatus};

/// Station progress to reaching the destination.
///
/// This is a high level item that is included in the user facing progress
/// report.
#[derive(Clone, Debug)]
pub struct StationProgress {
    /// Whether this station has been visited.
    pub visit_status: VisitStatus,
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
    pub fn new<E>(station_spec: &StationSpec<E>, progress_limit: ProgressLimit) -> Self {
        let visit_status = VisitStatus::NotReady;
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
            visit_status,
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

    /// Updates the style of the progress bar.
    pub fn progress_style_update(&self) {
        let progress_length = match self.progress_limit {
            ProgressLimit::Unknown => 0, // indicatif uses `0` for spinner type progress bars.
            ProgressLimit::Steps(n) | ProgressLimit::Bytes(n) => n,
        };

        let progress_style_template =
            Self::progress_style_template(self.visit_status, self.progress_limit);
        self.progress_bar.set_length(progress_length);

        self.progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(progress_style_template.as_str())
                .progress_chars(StationProgress::PROGRESS_CHARS),
        );

        // Redraw the progress bar
        self.progress_bar.tick();
    }

    fn progress_style_template(visit_status: VisitStatus, progress_limit: ProgressLimit) -> String {
        let (symbol, status) = match visit_status {
            VisitStatus::NotReady => ("🎫", "not ready"),
            VisitStatus::ParentFail => ("☠️ ", "parent fail"), // Extra space is deliberate
            VisitStatus::Queued => ("⏳", "queued"),
            VisitStatus::CheckFail => ("❌", "check fail"),
            VisitStatus::InProgress => ("{spinner:.green}{spinner:.green}", "in progress"),
            VisitStatus::VisitUnnecessary => ("✅", "visit unnecessary"),
            VisitStatus::VisitSuccess => ("✅", "visit success"),
            VisitStatus::VisitFail => ("❌", "visit fail"),
        };

        let progress_bar = if progress_limit == ProgressLimit::Unknown {
            console::style("▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒").blue()
        } else {
            let progress_bar = match visit_status {
                VisitStatus::NotReady => "{bar:40.blue.dim/blue}",
                VisitStatus::ParentFail => "{bar:40.red/red.dim}",
                VisitStatus::Queued => "{bar:40.blue.dim/blue}",
                VisitStatus::CheckFail => "{bar:40.black.bright/red}",
                VisitStatus::InProgress => "{bar:40.blue.dim/blue}",
                VisitStatus::VisitUnnecessary => "{bar:40.green.dim/green}",
                VisitStatus::VisitSuccess => "{bar:40.green/green}",
                VisitStatus::VisitFail => "{bar:40.black.bright/red}",
            };
            // Hack to return the same type.
            console::style(progress_bar)
        };

        let units = match progress_limit {
            ProgressLimit::Unknown => "",
            ProgressLimit::Steps(_) => "{pos}/{len}",
            ProgressLimit::Bytes(_) => "{bytes}/{total_bytes}",
        };

        format!(
            "{symbol} {{msg:20}} [{progress_bar}] {units} ({status})",
            symbol = symbol,
            progress_bar = progress_bar,
            units = units,
            status = status,
        )
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
