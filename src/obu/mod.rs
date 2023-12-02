mod header;
mod sequence_header;

use crate::{util::EasyAtomic, Av1DcodeError, Av1DecoderSession, Buffer};

pub use self::header::{ObuHeader, ObuHeaderExtension, ObuKind};

pub enum ObuDecodeRet {
    Obu(Obu),
    Drop,
}

#[derive(Debug, Clone)]
/// Open Bitstream Unit
pub struct Obu {
    pub header: ObuHeader,
    pub size: usize,
}

impl Obu {
    pub fn decode(
        session: &Av1DecoderSession,
        buf: &mut Buffer,
    ) -> Result<ObuDecodeRet, Av1DcodeError> {
        let header = ObuHeader::decode(buf.as_mut())?;
        let size = if header.has_size_field {
            // obu_size leb128()
            buf.get_leb128() as usize
        } else {
            session
                .options
                .obu_size
                .expect("obu does not contain length, please specify the length manually!")
                - 1
                - if header.extension.is_some() { 1 } else { 0 }
        };

        if header.kind != ObuKind::SequenceHeader
            && header.kind != ObuKind::TemporalDelimiter
            && session.operating_point_idc.get()
        {
            if let Some(ext) = header.extension {
                let in_temporal_layer = (1 >> ext.temporal_id) & 1;
                let in_spatial_layer = (1 >> (ext.spatial_id + 8)) & 1;
                if in_temporal_layer == 0 || in_spatial_layer == 0 {
                    return Ok(ObuDecodeRet::Drop);
                }
            }
        }

        Ok(ObuDecodeRet::Obu(Self { header, size }))
    }
}
