use std::{
    fmt,
    io::{self, Write},
    marker::PhantomData,
};

use choochoo_cfg_model::VisitStatus;
use choochoo_rt_model::{
    error::AsDiagnostic,
    srcerr::codespan_reporting::{term, term::termcolor::Buffer},
    Destination, Files, RwFiles, TrainReport,
};
use futures::{stream, StreamExt, TryStreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

/// Format trait for plain text.
#[derive(Debug)]
pub struct PlainTextFormatter<W, E>(PhantomData<(W, E)>);

struct WriterAndBuffer<'w, W> {
    writer: BufWriter<&'w mut W>,
    buffer: Vec<u8>,
}

impl<'w, W> WriterAndBuffer<'w, W>
where
    W: AsyncWrite + Unpin,
{
    fn new(w: &'w mut W) -> Self {
        // For `W: AsyncWrite`, we cannot use `write!` because `write!` expects `W:
        // std::io::Write`.
        //
        // To avoid a string allocation per `format!()` call, we preallocate one
        // `String`, and use it as our in-memory buffer.
        Self {
            writer: BufWriter::new(w),
            buffer: Vec::with_capacity(1024),
        }
    }
}

macro_rules! b_write_bytes {
    ($writer_and_buffer:ident, $byte_slice:expr) => {
        std::io::Write::write_all(&mut $writer_and_buffer.buffer, $byte_slice)?;
        AsyncWriteExt::write(&mut $writer_and_buffer.writer, &$writer_and_buffer.buffer).await?;
        $writer_and_buffer.buffer.clear();
    };
}

macro_rules! b_writeln {
    ($writer_and_buffer:ident) => {
        writeln!($writer_and_buffer.buffer)?;
        AsyncWriteExt::write(&mut $writer_and_buffer.writer, &$writer_and_buffer.buffer).await?;
        $writer_and_buffer.buffer.clear();
    };

    ($writer_and_buffer:ident, $fmt_str:expr, $($fmt_args:tt)*) => {
        writeln!($writer_and_buffer.buffer, $fmt_str, $($fmt_args)*)?;
        AsyncWriteExt::write(&mut $writer_and_buffer.writer, &$writer_and_buffer.buffer).await?;
        $writer_and_buffer.buffer.clear();
    };
}

impl<W, E> PlainTextFormatter<W, E>
where
    W: AsyncWrite + Unpin,
    E: AsDiagnostic<'static, Files = Files> + fmt::Debug + Send + Sync + 'static,
{
    /// Formats the value using the given formatter.
    pub async fn fmt(
        w: &mut W,
        dest: &Destination<E>,
        train_report: &TrainReport<E>,
    ) -> Result<(), io::Error> {
        let mut write_buf = WriterAndBuffer::new(w);
        write_buf = Self::write_station_statuses(dest, write_buf).await?;

        // `E` should either:
        //
        // * Be a `codespan_reporting::diagnostic::Diagnostic` which means we need to
        //   store the `Files<'a>` that the diagnostic's `file_id` comes from separately
        //   (maybe in `TrainReport`, or in the `Station` somehow), or
        //
        // * It should store its own `SimpleFile`, and we call `term::emit` with that
        //   (and we retrieve `files` from E itself).
        let writer = Buffer::ansi(); // TODO: switch between `ansi()` and `no_color()`
        let config = term::Config::default();
        let config = &config;
        let files = &*train_report.borrow::<RwFiles>();
        let files = files.read().await;
        let files = &*files;

        let station_errors = train_report.station_errors();
        let station_rt_id_to_error = station_errors.read().await;
        let (mut write_buf, _writer) = stream::iter(station_rt_id_to_error.values())
            .map(Result::<&E, io::Error>::Ok)
            .try_fold(
                (write_buf, writer),
                |(mut write_buf, mut writer), error| async move {
                    let diagnostic = error.as_diagnostic(files);

                    term::emit(&mut writer, config, files, &diagnostic)
                        .expect("TODO: Handle codespan_reporting::files::Error");
                    b_write_bytes!(write_buf, writer.as_slice());

                    Ok((write_buf, writer))
                },
            )
            .await?;

        write_buf.writer.flush().await
    }

    /// Formats the errors using the given formatter.
    pub async fn fmt_errors(w: &mut W, train_report: &TrainReport<E>) -> Result<(), io::Error> {
        let write_buf = WriterAndBuffer::new(w);

        // `E` should either:
        //
        // * Be a `codespan_reporting::diagnostic::Diagnostic` which means we need to
        //   store the `Files<'a>` that the diagnostic's `file_id` comes from separately
        //   (maybe in `TrainReport`, or in the `Station` somehow), or
        //
        // * It should store its own `SimpleFile`, and we call `term::emit` with that
        //   (and we retrieve `files` from E itself).
        let writer = Buffer::ansi(); // TODO: switch between `ansi()` and `no_color()`
        let config = term::Config::default();
        let config = &config;
        let files = &*train_report.borrow::<RwFiles>();
        let files = files.read().await;
        let files = &*files;

        let station_errors = train_report.station_errors();
        let station_rt_id_to_error = station_errors.read().await;
        let (mut write_buf, _writer) = stream::iter(station_rt_id_to_error.values())
            .map(Result::<&E, io::Error>::Ok)
            .try_fold(
                (write_buf, writer),
                |(mut write_buf, mut writer), error| async move {
                    let diagnostic = error.as_diagnostic(files);

                    term::emit(&mut writer, config, files, &diagnostic)
                        .expect("TODO: Handle codespan_reporting::files::Error");
                    b_write_bytes!(write_buf, writer.as_slice());

                    Ok((write_buf, writer))
                },
            )
            .await?;

        write_buf.writer.flush().await
    }

    // clippy warns on this, but if we elide the lifetime, it doesn't compile.
    #[allow(clippy::needless_lifetimes)]
    async fn write_station_statuses<'w>(
        dest: &Destination<E>,
        write_buf: WriterAndBuffer<'w, W>,
    ) -> Result<WriterAndBuffer<'w, W>, io::Error> {
        stream::iter(dest.stations())
            .map(Result::<_, io::Error>::Ok)
            .try_fold(write_buf, |mut write_buf, station| async move {
                let icon = match station.progress.visit_status {
                    VisitStatus::ParentPending => "⏰",
                    VisitStatus::ParentFail => "☠️",
                    VisitStatus::Queued => "⏳",
                    VisitStatus::InProgress => "⏳",
                    VisitStatus::VisitUnnecessary | VisitStatus::VisitSuccess => "✅",
                    VisitStatus::SetupFail | VisitStatus::CheckFail | VisitStatus::VisitFail => {
                        "❌"
                    }
                };

                b_writeln!(
                    write_buf,
                    "{status} {name}: {desc}",
                    status = icon,
                    name = station.spec.name(),
                    desc = station.spec.description()
                );
                Ok(write_buf)
            })
            .await
    }
}
