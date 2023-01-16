use std::{io::ErrorKind, path::PathBuf, sync::Arc};

use bytes::{Buf, BufMut, BytesMut};
use notify::{
    event::{
        AccessKind, AccessMode, CreateKind, DataChange, Event, EventAttributes, MetadataKind,
        ModifyKind, RemoveKind, RenameMode,
    },
    EventKind,
};
use tokio_util::codec::{Decoder, Encoder};

pub struct WatcherEventEncoder;

impl Encoder<&Event> for WatcherEventEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: &Event, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Write event kind.
        match &item.kind {
            EventKind::Any => dst.put_u8(0),
            EventKind::Access(access_kind) => {
                dst.put_u8(1);

                match access_kind {
                    AccessKind::Any => dst.put_u8(0),
                    AccessKind::Read => dst.put_u8(1),
                    AccessKind::Open(access_mode) => {
                        dst.put_u8(2);

                        match access_mode {
                            AccessMode::Any => dst.put_u8(0),
                            AccessMode::Execute => dst.put_u8(1),
                            AccessMode::Read => dst.put_u8(2),
                            AccessMode::Write => dst.put_u8(3),
                            AccessMode::Other => dst.put_u8(4),
                        }
                    }
                    AccessKind::Close(access_mode) => {
                        dst.put_u8(3);

                        match access_mode {
                            AccessMode::Any => dst.put_u8(0),
                            AccessMode::Execute => dst.put_u8(1),
                            AccessMode::Read => dst.put_u8(2),
                            AccessMode::Write => dst.put_u8(3),
                            AccessMode::Other => dst.put_u8(4),
                        }
                    }
                    AccessKind::Other => dst.put_u8(4),
                }
            }
            EventKind::Create(create_kind) => {
                dst.put_u8(2);

                match create_kind {
                    CreateKind::Any => dst.put_u8(0),
                    CreateKind::File => dst.put_u8(1),
                    CreateKind::Folder => dst.put_u8(2),
                    CreateKind::Other => dst.put_u8(3),
                }
            }
            EventKind::Modify(modify_kind) => {
                dst.put_u8(3);

                match modify_kind {
                    ModifyKind::Any => dst.put_u8(0),
                    ModifyKind::Data(data_change) => {
                        dst.put_u8(1);

                        match data_change {
                            DataChange::Any => dst.put_u8(0),
                            DataChange::Size => dst.put_u8(1),
                            DataChange::Content => dst.put_u8(2),
                            DataChange::Other => dst.put_u8(3),
                        }
                    }
                    ModifyKind::Metadata(metadata_kind) => {
                        dst.put_u8(2);

                        match metadata_kind {
                            MetadataKind::Any => dst.put_u8(0),
                            MetadataKind::AccessTime => dst.put_u8(1),
                            MetadataKind::WriteTime => dst.put_u8(2),
                            MetadataKind::Permissions => dst.put_u8(3),
                            MetadataKind::Ownership => dst.put_u8(4),
                            MetadataKind::Extended => dst.put_u8(5),
                            MetadataKind::Other => dst.put_u8(6),
                        }
                    }
                    ModifyKind::Name(rename_mode) => {
                        dst.put_u8(3);

                        match rename_mode {
                            RenameMode::Any => dst.put_u8(0),
                            RenameMode::To => dst.put_u8(1),
                            RenameMode::From => dst.put_u8(2),
                            RenameMode::Both => dst.put_u8(3),
                            RenameMode::Other => dst.put_u8(4),
                        }
                    }
                    ModifyKind::Other => dst.put_u8(4),
                }
            }
            EventKind::Remove(remove_kind) => {
                dst.put_u8(4);

                match remove_kind {
                    RemoveKind::Any => dst.put_u8(0),
                    RemoveKind::File => dst.put_u8(1),
                    RemoveKind::Folder => dst.put_u8(2),
                    RemoveKind::Other => dst.put_u8(3),
                }
            }
            EventKind::Other => dst.put_u8(5),
        }

        // Write paths.
        let number_of_paths = item.paths.len() as u8;
        dst.put_u8(number_of_paths);

        for path in &item.paths {
            let path = path.to_str().ok_or_else(|| {
                std::io::Error::new(ErrorKind::Other, format!("Invalid path: {path:?}"))
            })?;
            let path_len = path.len() as u16;

            dst.put_u16_le(path_len);
            dst.put(path.as_bytes());
        }

        // TODO: Maybe write attributes.

        Ok(())
    }
}

pub struct WatcherEventDecoder;

impl Decoder for WatcherEventDecoder {
    type Item = Event;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Read event kind.
        if src.len() < 3 {
            src.reserve(3_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let kind = src.get_u8();
        let kind = match kind {
            0 => EventKind::Any,
            1 => {
                let access_kind = src.get_u8();
                let access_kind = match access_kind {
                    0 => AccessKind::Any,
                    1 => AccessKind::Read,
                    2 => {
                        let access_mode = src.get_u8();
                        let access_mode = match access_mode {
                            0 => AccessMode::Any,
                            1 => AccessMode::Execute,
                            2 => AccessMode::Read,
                            3 => AccessMode::Write,
                            4 => AccessMode::Other,

                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid event access kind: {access_kind}"),
                                ));
                            }
                        };

                        AccessKind::Open(access_mode)
                    }
                    3 => {
                        let access_mode = src.get_u8();
                        let access_mode = match access_mode {
                            0 => AccessMode::Any,
                            1 => AccessMode::Execute,
                            2 => AccessMode::Read,
                            3 => AccessMode::Write,
                            4 => AccessMode::Other,

                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid event access kind: {access_kind}"),
                                ));
                            }
                        };

                        AccessKind::Close(access_mode)
                    }
                    4 => AccessKind::Other,

                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid event access kind: {access_kind}"),
                        ));
                    }
                };

                EventKind::Access(access_kind)
            }
            2 => {
                let create_kind = src.get_u8();
                let create_kind = match create_kind {
                    0 => CreateKind::Any,
                    1 => CreateKind::File,
                    2 => CreateKind::Folder,
                    3 => CreateKind::Other,

                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid event create kind: {create_kind}"),
                        ));
                    }
                };

                EventKind::Create(create_kind)
            }
            3 => {
                let modify_kind = src.get_u8();
                let modify_kind = match modify_kind {
                    0 => ModifyKind::Any,
                    1 => {
                        let data_change = src.get_u8();
                        let data_change = match data_change {
                            0 => DataChange::Any,
                            1 => DataChange::Size,
                            2 => DataChange::Content,
                            3 => DataChange::Other,

                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid event modify kind data change: {data_change}"),
                                ));
                            }
                        };

                        ModifyKind::Data(data_change)
                    }
                    2 => {
                        let metadata_kind = src.get_u8();
                        let metadata_kind = match metadata_kind {
                            0 => MetadataKind::Any,
                            1 => MetadataKind::AccessTime,
                            2 => MetadataKind::WriteTime,
                            3 => MetadataKind::Permissions,
                            4 => MetadataKind::Ownership,
                            5 => MetadataKind::Extended,
                            6 => MetadataKind::Other,

                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid event modify metadata kind: {metadata_kind}"),
                                ));
                            }
                        };

                        ModifyKind::Metadata(metadata_kind)
                    }
                    3 => {
                        let rename_mode = src.get_u8();
                        let rename_mode = match rename_mode {
                            0 => RenameMode::Any,
                            1 => RenameMode::To,
                            2 => RenameMode::From,
                            3 => RenameMode::Both,
                            4 => RenameMode::Other,

                            _ => {
                                return Err(std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid event modify kind rename mode: {rename_mode}"),
                                ));
                            }
                        };

                        ModifyKind::Name(rename_mode)
                    }
                    4 => ModifyKind::Other,

                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid event modify kind: {modify_kind}"),
                        ));
                    }
                };

                EventKind::Modify(modify_kind)
            }
            4 => {
                let remove_kind = src.get_u8();
                let remove_kind = match remove_kind {
                    0 => RemoveKind::Any,
                    1 => RemoveKind::File,
                    2 => RemoveKind::Folder,
                    3 => RemoveKind::Other,

                    _ => {
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid event remove kind: {remove_kind}"),
                        ));
                    }
                };

                EventKind::Remove(remove_kind)
            }
            5 => EventKind::Other,

            _ => {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid event kind: {kind}"),
                ))
            }
        };

        // Read paths.
        if src.is_empty() {
            src.reserve(1_usize.saturating_sub(src.len()));

            return Ok(None);
        }

        let number_of_paths = src.get_u8();
        let mut paths = Vec::with_capacity(number_of_paths as usize);
        for _ in 0..number_of_paths {
            if src.len() < 2 {
                src.reserve(2_usize.saturating_sub(src.len()));

                return Ok(None);
            }

            let path_len = src.get_u16_le() as usize;
            if src.len() < path_len {
                src.reserve(path_len.saturating_sub(src.len()));

                return Ok(None);
            }

            let path = src.split_to(path_len);
            let path = path.to_vec();
            let path = String::from_utf8(path).map_err(|e| {
                std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Fail to parse string: {e:?}"),
                )
            })?;
            let path = PathBuf::from(path);

            paths.push(path);
        }

        // Return object.
        Ok(Some(Event {
            kind,
            paths,
            attrs: EventAttributes::new(), // TODO: Maybe read attributes.
        }))
    }
}
