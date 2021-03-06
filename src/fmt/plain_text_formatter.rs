use std::{
    io::{self, Write},
    marker::PhantomData,
};

use futures::{stream, StreamExt, TryStreamExt};
use srcerr::codespan_reporting::{term, term::termcolor::Buffer};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::rt_model::{
    error::AsDiagnostic, Destination, Files, RwFiles, Station, Stations, TrainReport, VisitStatus,
};

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
    E: AsDiagnostic<'static, Files = Files>,
{
    /// Formats the value using the given formatter.
    pub async fn fmt(
        w: &mut W,
        dest: &Destination<E>,
        train_report: &TrainReport<E>,
    ) -> Result<(), io::Error> {
        let mut write_buf = WriterAndBuffer::new(w);
        write_buf = Self::write_station_statuses(&dest.stations, write_buf).await?;

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
        let files = &*train_report.resources.borrow::<RwFiles>();
        let files = files.read().await;
        let files = &*files;

        let (mut write_buf, _writer) = stream::iter(train_report.errors.values())
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
        let files = &*train_report.resources.borrow::<RwFiles>();
        let files = files.read().await;
        let files = &*files;

        let (mut write_buf, _writer) = stream::iter(train_report.errors.values())
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
        stations: &Stations<E>,
        write_buf: WriterAndBuffer<'w, W>,
    ) -> Result<WriterAndBuffer<'w, W>, io::Error> {
        stream::iter(stations.iter())
            .map(Result::<&Station<E>, io::Error>::Ok)
            .try_fold(write_buf, |mut write_buf, station| async move {
                let station_spec = &station.station_spec;
                let icon = match station.visit_status {
                    VisitStatus::NotReady => "⏰",
                    VisitStatus::ParentFail => "☠️",
                    VisitStatus::Queued => "⏳",
                    VisitStatus::InProgress => "⏳",
                    VisitStatus::VisitUnnecessary | VisitStatus::VisitSuccess => "✅",
                    VisitStatus::CheckFail | VisitStatus::VisitFail => "❌",
                };

                b_writeln!(
                    write_buf,
                    "{status} {name}: {desc}",
                    status = icon,
                    name = station_spec.name(),
                    desc = station_spec.description()
                );
                Ok(write_buf)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use daggy::{petgraph::graph::DefaultIx, NodeIndex};
    use tokio::runtime;

    use super::PlainTextFormatter;
    use crate::{
        cfg_model::{StationFn, StationId, StationIdInvalidFmt, StationSpec, StationSpecFns},
        rt_model::{Destination, Station, Stations, TrainReport, VisitStatus},
    };

    #[test]
    fn writes_station_status_name_and_description() -> Result<(), Box<dyn std::error::Error>> {
        let rt = runtime::Builder::new_current_thread().build()?;
        let mut output = Vec::with_capacity(1024);
        let dest = {
            let mut stations = Stations::new();
            add_station(&mut stations, "a", "A", "a_desc", VisitStatus::NotReady)?;
            add_station(&mut stations, "b", "B", "b_desc", VisitStatus::ParentFail)?;
            add_station(&mut stations, "c", "C", "c_desc", VisitStatus::Queued)?;
            add_station(&mut stations, "d", "D", "d_desc", VisitStatus::InProgress)?;
            add_station(&mut stations, "e", "E", "e_desc", VisitStatus::VisitSuccess)?;
            add_station(
                &mut stations,
                "f",
                "F",
                "f_desc",
                VisitStatus::VisitUnnecessary,
            )?;
            add_station(&mut stations, "g", "G", "g_desc", VisitStatus::VisitFail)?;
            add_station(&mut stations, "h", "H", "h_desc", VisitStatus::CheckFail)?;
            Destination { stations }
        };
        let train_report = TrainReport::new();

        rt.block_on(PlainTextFormatter::fmt(&mut output, &dest, &train_report))?;

        assert_eq!(
            "\
            ⏰ A: a_desc\n\
            ☠️ B: b_desc\n\
            ⏳ C: c_desc\n\
            ⏳ D: d_desc\n\
            ✅ E: e_desc\n\
            ✅ F: f_desc\n\
            ❌ G: g_desc\n\
            ❌ H: h_desc\n\
            ",
            String::from_utf8(output)?
        );

        Ok(())
    }

    fn add_station(
        stations: &mut Stations<()>,
        station_id: &'static str,
        station_name: &'static str,
        station_desc: &'static str,
        visit_status: VisitStatus,
    ) -> Result<NodeIndex<DefaultIx>, StationIdInvalidFmt<'static>> {
        let station_id = StationId::new(station_id)?;
        let station_spec_fns = {
            let visit_fn = StationFn::new(|_, _| Box::pin(async { Result::<(), ()>::Ok(()) }));
            StationSpecFns::new(visit_fn)
        };
        let station_spec = StationSpec::new(
            station_id,
            String::from(station_name),
            String::from(station_desc),
            station_spec_fns,
        );
        let station = Station::new(station_spec, visit_status);
        Ok(stations.add_node(station))
    }
}
