use std::{borrow::Cow, path::Path};

use choochoo::{
    cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns, Workload},
    cli_fmt::PlainTextFormatter,
    rt_logic::Train,
    rt_model::{
        error::StationSpecError,
        srcerr::{
            self,
            codespan::{FileId, Files, Span},
            codespan_reporting::diagnostic::{Diagnostic, Severity},
            SourceError,
        },
        Destination, RwFiles,
    },
};
use tokio::{fs, runtime};

use crate::error::{ErrorCode, ErrorDetail};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = runtime::Builder::new_current_thread().build()?;
    rt.block_on(async move {
        let mut dest = {
            let mut builder = Destination::builder();
            let [station_a, station_b] = builder.add_stations([station_a()?, station_b()?]);
            builder.add_edge(station_a, station_b, Workload::default())?;

            let dest = builder.build();

            dest
        };
        let train_report = Train::reach(&mut dest).await?;

        let mut stdout = tokio::io::stdout();

        PlainTextFormatter::fmt(&mut stdout, &dest, &train_report).await?;

        Result::<(), Box<dyn std::error::Error>>::Ok(())
    })?;

    Ok(())
}

#[derive(Debug)]
pub struct ExampleError(pub SourceError<'static, ErrorCode, ErrorDetail, Files<Cow<'static, str>>>);

impl choochoo::rt_model::error::AsDiagnostic<'static> for ExampleError {
    type Files = Files<Cow<'static, str>>;

    fn as_diagnostic(
        &self,
        files: &Self::Files,
    ) -> Diagnostic<<Self::Files as srcerr::codespan_reporting::files::Files<'static>>::FileId>
    {
        SourceError::as_diagnostic(&self.0, files)
    }
}

impl From<StationSpecError> for ExampleError {
    fn from(error: StationSpecError) -> ExampleError {
        let code = ErrorCode::StationSpecError;
        let detail = ErrorDetail::StationSpecError(error);

        ExampleError(SourceError::new(code, detail, Severity::Bug))
    }
}

fn station_a() -> Result<StationSpec<ExampleError>, StationIdInvalidFmt<'static>> {
    new_station(
        "a",
        "Station A",
        "Prints visit.",
        StationFn::new(|_station, _| {
            Box::pin(async move {
                eprintln!("Visiting {}.", "Station A");
                Result::<(), ExampleError>::Ok(())
            })
        }),
    )
}

fn station_b() -> Result<StationSpec<ExampleError>, StationIdInvalidFmt<'static>> {
    new_station(
        "b",
        "Station B",
        "Reads `simple.toml` and reports error.",
        StationFn::new(move |_station, resources| {
            Box::pin(async move {
                eprintln!("Visiting {}.", "Station B");

                let files = resources.borrow_mut::<RwFiles>();
                let mut files = files.write().await;

                let file_id = read_simple_toml(&mut files)
                    .await
                    .expect("Failed to read simple.toml");

                let error = value_out_of_range(file_id);
                Result::<(), ExampleError>::Err(error)
            })
        }),
    )
}

async fn read_simple_toml(files: &mut Files<Cow<'static, str>>) -> Result<FileId, std::io::Error> {
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/simple.toml"));
    let content = fs::read(path).await?;
    let content = String::from_utf8(content).expect("Expected simple.toml to be UTF8.");

    let path_display = path.display().to_string();
    let file_id = files.add(path_display.as_str(), Cow::Owned(content));

    Ok(file_id)
}

fn value_out_of_range(file_id: FileId) -> ExampleError {
    let error_code = ErrorCode::ValueOutOfRange;
    let error_detail = ErrorDetail::ValueOutOfRange {
        file_id,
        value: -1,
        value_byte_indices: Span::from(21..23),
        range: 1..=3,
    };
    let severity = Severity::Error;

    ExampleError(SourceError::new(error_code, error_detail, severity))
}

fn new_station(
    station_id: &'static str,
    station_name: &'static str,
    station_description: &'static str,
    visit_fn: StationFn<(), ExampleError>,
) -> Result<StationSpec<ExampleError>, StationIdInvalidFmt<'static>> {
    let station_id = StationId::new(station_id)?;
    let station_name = String::from(station_name);
    let station_description = String::from(station_description);
    let station_spec_fns = StationSpecFns::new(visit_fn);
    Ok(StationSpec::new(
        station_id,
        station_name,
        station_description,
        station_spec_fns,
    ))
}

mod error {
    use std::{borrow::Cow, ops::RangeInclusive};

    use choochoo::rt_model::{
        error::StationSpecError,
        srcerr::{
            self,
            codespan::{FileId, Files, Span},
            codespan_reporting::diagnostic::Label,
            fmt::Note,
        },
    };

    /// Error codes for simple example.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ErrorCode {
        /// Error when a value is out of range.
        ValueOutOfRange,
        /// There is a bug with the station specification.
        StationSpecError,
    }

    impl srcerr::ErrorCode for ErrorCode {
        const ERROR_CODE_MAX: usize = 2;
        const PREFIX: &'static str = "E";

        fn code(self) -> usize {
            match self {
                Self::ValueOutOfRange => 1,
                Self::StationSpecError => 2,
            }
        }

        fn description(self) -> &'static str {
            match self {
                Self::ValueOutOfRange => "Value out of range.",
                Self::StationSpecError => "There is a bug with the station specification.",
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
        /// There is a bug with the station specification.
        StationSpecError(StationSpecError),
    }

    impl srcerr::ErrorDetail<'static> for ErrorDetail {
        type Files = Files<Cow<'static, str>>;

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
                Self::StationSpecError(_error) => vec![],
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
                Self::StationSpecError(error) => vec![
                    String::from(
                        "Make sure the `visit_fn` updates what the `check_fn` is reading.",
                    ),
                    error.to_string(),
                ],
            }
        }
    }
}
