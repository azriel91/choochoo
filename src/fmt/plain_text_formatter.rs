use std::{
    io::{self, Write},
    marker::PhantomData,
};

use futures::{stream, StreamExt, TryStreamExt};
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::rt_model::{Destination, Station, VisitStatus};

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

macro_rules! write_b {
    ($writer_and_buffer:ident, $fmt_str:expr, $($fmt_args:tt)*) => {
        write!($writer_and_buffer.buffer, $fmt_str, $($fmt_args)*)?;
        AsyncWriteExt::write(&mut $writer_and_buffer.writer, &$writer_and_buffer.buffer).await?;
        $writer_and_buffer.buffer.clear();
    };
}

impl<W, E> PlainTextFormatter<W, E>
where
    W: AsyncWrite + Unpin,
{
    /// Formats the value using the given formatter.
    pub async fn fmt(w: &mut W, dest: &Destination<E>) -> Result<(), io::Error> {
        let mut write_buf = WriterAndBuffer::new(w);
        write_buf = stream::iter(dest.stations.iter())
            .map(Result::<&Station<E>, io::Error>::Ok)
            .try_fold(write_buf, |mut write_buf, station| async move {
                let station_spec = &station.station_spec;
                let icon = match station.visit_status {
                    VisitStatus::NotReady => "⏰",
                    VisitStatus::ParentFail => "☠️",
                    VisitStatus::Queued => "⏳",
                    VisitStatus::InProgress => "⏳",
                    VisitStatus::VisitSuccess => "✅",
                    VisitStatus::VisitFail => "❌",
                };

                write_b!(
                    write_buf,
                    "{status} {name}: {desc}\n",
                    status = icon,
                    name = station_spec.name(),
                    desc = station_spec.description()
                );

                Ok(write_buf)
            })
            .await?;

        write_buf.writer.flush().await
    }
}

#[cfg(test)]
mod tests {
    use daggy::{petgraph::graph::DefaultIx, NodeIndex};
    use tokio::runtime;

    use super::PlainTextFormatter;
    use crate::{
        cfg_model::{StationId, StationIdInvalidFmt, StationSpec, VisitFn},
        rt_model::{Destination, Station, Stations, VisitStatus},
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
            add_station(&mut stations, "f", "F", "f_desc", VisitStatus::VisitFail)?;
            Destination { stations }
        };

        rt.block_on(PlainTextFormatter::fmt(&mut output, &dest))?;

        assert_eq!(
            "\
            ⏰ A: a_desc\n\
            ☠️ B: b_desc\n\
            ⏳ C: c_desc\n\
            ⏳ D: d_desc\n\
            ✅ E: e_desc\n\
            ❌ F: f_desc\n\
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
        let visit_fn = VisitFn::new(|_station| Box::pin(async move { Result::<(), ()>::Ok(()) }));
        let station_spec = StationSpec::new(
            station_id,
            String::from(station_name),
            String::from(station_desc),
            visit_fn,
        );
        let station = Station::new(station_spec, visit_status);
        Ok(stations.add_node(station))
    }
}
