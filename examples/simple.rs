use std::{borrow::Cow, path::Path};

use choochoo::{
    cfg_model::{StationId, StationIdInvalidFmt, StationSpec, VisitFn},
    fmt::PlainTextFormatter,
    rt_model::{Destination, Station, Stations, VisitStatus},
    Train,
};
use codespan::{FileId, Files, Span};
use daggy::{petgraph::graph::DefaultIx, NodeIndex};

use srcerr::{codespan_reporting::diagnostic::Severity, SourceError};
use tokio::{fs, runtime};

use crate::error::{ErrorCode, ErrorDetail};

type ExampleError<'f> = SourceError<'f, ErrorCode, ErrorDetail, Files<Cow<'f, str>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    let mut files = Files::<Cow<'_, str>>::new();

    rt.block_on(async move {
        let file_id = read_simple_toml(&mut files)
            .await
            .expect("Failed to read simple.toml");

        let (mut dest, _station_a, _station_b) = {
            let mut stations = Stations::new();
            let station_a = station_a(&mut stations);
            let station_b = station_b(&mut stations, file_id);
            let dest = Destination { stations };

            (dest, station_a, station_b)
        };
        let mut train_report = Train::reach(&mut dest).await;
        // Hack: We need the station to have access to `Files`.
        train_report.files = files;

        let mut stdout = tokio::io::stdout();

        PlainTextFormatter::fmt(&mut stdout, &dest, &train_report)
            .await
            .expect("Failed to format train report.");
    });

    Ok(())
}

async fn read_simple_toml<'files>(
    files: &mut Files<Cow<'files, str>>,
) -> Result<FileId, std::io::Error> {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/simple.toml"));
    let content = fs::read(path).await?;
    let content = String::from_utf8(content).expect("Expected simple.toml to be UTF8.");

    let path_display = path.display().to_string();
    let file_id = files.add(path_display.as_str(), Cow::Owned(content));

    Ok(file_id)
}

fn station_a(
    stations: &mut Stations<ExampleError<'_>>,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let visit_fn = VisitFn::new(|station| {
        Box::pin(async move {
            eprintln!("Visiting {}.", station.station_spec.name());
            Result::<(), ExampleError<'_>>::Ok(())
        })
    });
    add_station(
        stations,
        "a",
        "Station A",
        "Prints visit.",
        VisitStatus::Queued,
        visit_fn,
    )
}

fn station_b(
    stations: &mut Stations<ExampleError<'_>>,
    file_id: FileId,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let visit_fn = VisitFn::new(move |station| {
        Box::pin(async move {
            eprintln!("Visiting {}.", station.station_spec.name());
            let error = value_out_of_range(file_id);
            Result::<(), ExampleError<'_>>::Err(error)
        })
    });
    add_station(
        stations,
        "b",
        "Station B",
        "Reads `simple.toml` and reports error.",
        VisitStatus::Queued,
        visit_fn,
    )
}

fn value_out_of_range<'f>(
    file_id: FileId,
) -> SourceError<'f, ErrorCode, ErrorDetail, Files<Cow<'f, str>>> {
    let error_code = ErrorCode::ValueOutOfRange;
    let error_detail = ErrorDetail::ValueOutOfRange {
        file_id,
        value: -1,
        value_byte_indices: Span::from(21..23),
        range: 1..=3,
    };
    let severity = Severity::Error;

    SourceError::new(error_code, error_detail, severity)
}

fn add_station<'files>(
    stations: &mut Stations<ExampleError<'files>>,
    station_id: &'static str,
    station_name: &'static str,
    station_description: &'static str,
    visit_status: VisitStatus,
    visit_fn: VisitFn<ExampleError<'files>>,
) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
    let station_id = StationId::new(station_id)?;
    let station_name = String::from(station_name);
    let station_description = String::from(station_description);
    let station_spec = StationSpec::new(station_id, station_name, station_description, visit_fn);
    let station = Station::new(station_spec, visit_status);
    Ok(stations.add_node(station))
}

mod error {
    use std::{borrow::Cow, ops::RangeInclusive};

    use codespan::{FileId, Files, Span};
    use srcerr::{codespan_reporting::diagnostic::Label, fmt::Note};

    /// Error codes for simple example.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ErrorCode {
        /// Error when a value is out of range.
        ValueOutOfRange,
    }

    impl srcerr::ErrorCode for ErrorCode {
        const ERROR_CODE_MAX: usize = 2;
        const PREFIX: &'static str = "E";

        fn code(self) -> usize {
            match self {
                Self::ValueOutOfRange => 1,
            }
        }

        fn description(self) -> &'static str {
            match self {
                Self::ValueOutOfRange => "Value out of range.",
            }
        }
    }

    /// Error detail for simple example.
    #[derive(Debug)]
    pub enum ErrorDetail {
        /// Error when a value is out of range.
        ValueOutOfRange {
            /// ID of the file containing the invalid value.
            file_id: FileId,
            /// The value.
            value: i32,
            /// Byte begin and end indices where the value is defined.
            value_byte_indices: Span,
            /// Range that the value must be within.
            range: RangeInclusive<u32>,
        },
    }

    impl<'files> srcerr::ErrorDetail<'files> for ErrorDetail {
        type Files = Files<Cow<'files, str>>;

        fn labels(&self) -> Vec<Label<FileId>> {
            match self {
                Self::ValueOutOfRange {
                    file_id,
                    value_byte_indices,
                    range,
                    ..
                } => {
                    vec![
                        Label::primary(*file_id, value_byte_indices.clone()).with_message(format!(
                            "not within the range: `{}..={}`",
                            range.start(),
                            range.end()
                        )),
                    ]
                }
            }
        }

        fn notes(&self, _files: &Self::Files) -> Vec<String> {
            match self {
                Self::ValueOutOfRange { range, .. } => {
                    let valid_exprs = range.clone().map(|n| Cow::Owned(n.to_string()));
                    let suggestion =
                        Note::valid_exprs(valid_exprs).expect("Failed to format note.");
                    vec![suggestion]
                }
            }
        }
    }
}
