use std::{fmt::Display, thread, time::Duration, collections::HashMap};

use libmpv::{Mpv, FileState, events::{Event, PropertyData}, Format, MpvNode};

pub enum Error {
    Mpv(libmpv::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Mpv(err) => err.to_string(),
            }
        )
    }
}

impl From<libmpv::Error> for Error {
    fn from(err: libmpv::Error) -> Self {
        Self::Mpv(err)
    }
}

pub struct Player {
    mpv: Mpv,
}

impl Player {
    pub fn new() -> Result<Self, Error> {
        let mpv = Mpv::new()?;
        Ok(Self { mpv })
    }

    pub async fn play(&self, url: &str) -> Result<(), Error> {
        //self.mpv.set_property("volume", 15)?;
        self.mpv.set_property("vo", "null")?;

        let mut ev_ctx = self.mpv.create_event_context();
        ev_ctx.disable_deprecated_events()?;
        ev_ctx.observe_property("volume", Format::Int64, 0)?;
        ev_ctx.observe_property("demuxer-cache-state", Format::Node, 0)?;

        crossbeam::scope(|scope| {
            scope.spawn(|_| {
                self.mpv.playlist_load_files(&[(&url, FileState::AppendPlay, None)])
                    .unwrap();


                // Trigger `Event::EndFile`.
                //self.mpv.playlist_next_force().unwrap();
            });

            scope.spawn(move |_| loop {
                let ev = ev_ctx.wait_event(600.).unwrap_or(Err(libmpv::Error::Null));

                match ev {
                    Ok(Event::EndFile(r)) => {
                        println!("Exiting! Reason: {:?}", r);
                        break;
                    }

                    Ok(Event::PropertyChange {
                        name: "demuxer-cache-state",
                        change: PropertyData::Node(mpv_node),
                        ..
                    }) => {
                        let ranges = seekable_ranges(mpv_node).unwrap();
                        println!("Seekable ranges updated: {:?}", ranges);
                    }
                    Ok(e) => println!("Event triggered: {:?}", e),
                    Err(e) => println!("Event errored: {:?}", e),
                }
            });
        })
        .unwrap();
        Ok(())
    }
}

fn seekable_ranges(demuxer_cache_state: &MpvNode) -> Option<Vec<(f64, f64)>> {
    let mut res = Vec::new();
    let props: HashMap<&str, MpvNode> = demuxer_cache_state.to_map()?.collect();
    let ranges = props.get("seekable-ranges")?.to_array()?;

    for node in ranges {
        let range: HashMap<&str, MpvNode> = node.to_map()?.collect();
        let start = range.get("start")?.to_f64()?;
        let end = range.get("end")?.to_f64()?;
        res.push((start, end));
    }

    Some(res)
}
