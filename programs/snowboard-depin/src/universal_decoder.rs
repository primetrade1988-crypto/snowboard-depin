use crate::constants::MAX_PAYLOAD_LEN;
use crate::errors::SnowboardDepinError;
use crate::state::{HardwareAdapter, PayloadFormat, TelemetrySample};
use anchor_lang::prelude::*;

/// Magic for BinaryV1: `SBD1` + packed TelemetrySample fields (no prefix).
pub const BINARY_MAGIC: &[u8; 4] = b"SBD1";

pub fn normalize(adapter: &HardwareAdapter, payload: &[u8]) -> Result<TelemetrySample> {
    require!(adapter.active, SnowboardDepinError::UnknownAdapter);
    require!(
        !payload.is_empty() && payload.len() <= MAX_PAYLOAD_LEN,
        SnowboardDepinError::InvalidPayload
    );
    match adapter.format {
        PayloadFormat::BinaryV1 => decode_binary_v1(adapter, payload),
        PayloadFormat::TaggedMap => decode_tagged_map(adapter, payload),
    }
}

fn decode_binary_v1(adapter: &HardwareAdapter, payload: &[u8]) -> Result<TelemetrySample> {
    require!(payload.len() >= 56, SnowboardDepinError::InvalidPayload);
    require!(
        &payload[0..4] == BINARY_MAGIC,
        SnowboardDepinError::DecodeFailed
    );
    Ok(TelemetrySample {
        nonce: read_u64(&payload[4..12])?,
        timestamp: read_i64(&payload[12..20])?,
        lat_e7: read_i32(&payload[20..24])?,
        lon_e7: read_i32(&payload[24..28])?,
        altitude_cm: read_i32(&payload[28..32])?,
        speed_cm_s: scale_u32(read_u32(&payload[32..36])?, adapter.speed_scale)?,
        accel_milli_g: read_u32(&payload[36..40])?,
        imu_delta: read_u32(&payload[40..44])?,
        distance_cm: scale_u32(read_u32(&payload[44..48])?, adapter.distance_scale)?,
        vertical_drop_cm: scale_u32(read_u32(&payload[48..52])?, adapter.drop_scale)?,
        airtime_ms: scale_u32(read_u32(&payload[52..56])?, adapter.airtime_scale)?,
    })
}

/// Compact tagged map: repeating `[key:u8][len:u8][value...]`.
/// Keys: 1 nonce u64, 2 ts i64, 3 lat i32, 4 lon i32, 5 alt i32,
/// 6 speed, 7 accel, 8 imu, 9 dist, 10 drop, 11 airtime.
fn decode_tagged_map(adapter: &HardwareAdapter, payload: &[u8]) -> Result<TelemetrySample> {
    let mut sample = TelemetrySample::default();
    let mut i = 0usize;
    while i + 2 <= payload.len() {
        let key = payload[i];
        let len = payload[i + 1] as usize;
        i += 2;
        require!(i + len <= payload.len(), SnowboardDepinError::InvalidPayload);
        let val = &payload[i..i + len];
        i += len;
        match key {
            1 => sample.nonce = read_int_le(val)?,
            2 => sample.timestamp = read_int_le(val)? as i64,
            3 => sample.lat_e7 = read_int_le(val)? as i32,
            4 => sample.lon_e7 = read_int_le(val)? as i32,
            5 => sample.altitude_cm = read_int_le(val)? as i32,
            6 => sample.speed_cm_s = scale_u32(read_int_le(val)? as u32, adapter.speed_scale)?,
            7 => sample.accel_milli_g = read_int_le(val)? as u32,
            8 => sample.imu_delta = read_int_le(val)? as u32,
            9 => sample.distance_cm = scale_u32(read_int_le(val)? as u32, adapter.distance_scale)?,
            10 => sample.vertical_drop_cm = scale_u32(read_int_le(val)? as u32, adapter.drop_scale)?,
            11 => sample.airtime_ms = scale_u32(read_int_le(val)? as u32, adapter.airtime_scale)?,
            _ => {}
        }
    }
    require!(sample.nonce > 0, SnowboardDepinError::DecodeFailed);
    Ok(sample)
}

fn scale_u32(raw: u32, scale: u32) -> Result<u32> {
    let s = if scale == 0 { 1 } else { scale };
    raw.checked_mul(s).ok_or(error!(SnowboardDepinError::MathOverflow))
}

fn read_u64(b: &[u8]) -> Result<u64> {
    require!(b.len() == 8, SnowboardDepinError::InvalidPayload);
    Ok(u64::from_le_bytes(b.try_into().unwrap()))
}

fn read_i64(b: &[u8]) -> Result<i64> {
    require!(b.len() == 8, SnowboardDepinError::InvalidPayload);
    Ok(i64::from_le_bytes(b.try_into().unwrap()))
}

fn read_u32(b: &[u8]) -> Result<u32> {
    require!(b.len() == 4, SnowboardDepinError::InvalidPayload);
    Ok(u32::from_le_bytes(b.try_into().unwrap()))
}

fn read_i32(b: &[u8]) -> Result<i32> {
    require!(b.len() == 4, SnowboardDepinError::InvalidPayload);
    Ok(i32::from_le_bytes(b.try_into().unwrap()))
}

fn read_int_le(b: &[u8]) -> Result<u64> {
    match b.len() {
        1 => Ok(b[0] as u64),
        2 => Ok(u16::from_le_bytes(b.try_into().unwrap()) as u64),
        4 => Ok(u32::from_le_bytes(b.try_into().unwrap()) as u64),
        8 => Ok(u64::from_le_bytes(b.try_into().unwrap())),
        _ => err!(SnowboardDepinError::DecodeFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::HardwareAdapter;

    fn adapter() -> HardwareAdapter {
        HardwareAdapter {
            admin: Pubkey::default(),
            brand_id: 1,
            format: PayloadFormat::BinaryV1,
            distance_scale: 1,
            speed_scale: 1,
            drop_scale: 1,
            airtime_scale: 1,
            active: true,
            bump: 0,
        }
    }

    #[test]
    fn binary_v1_roundtrip_fields() {
        let mut p = Vec::from(*BINARY_MAGIC);
        p.extend_from_slice(&7u64.to_le_bytes());
        p.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        p.extend_from_slice(&450_000_000i32.to_le_bytes());
        p.extend_from_slice(&(-1_200_000_000i32).to_le_bytes());
        p.extend_from_slice(&180_000i32.to_le_bytes());
        p.extend_from_slice(&800u32.to_le_bytes());
        p.extend_from_slice(&1200u32.to_le_bytes());
        p.extend_from_slice(&100u32.to_le_bytes());
        p.extend_from_slice(&25_000u32.to_le_bytes());
        p.extend_from_slice(&400u32.to_le_bytes());
        p.extend_from_slice(&1500u32.to_le_bytes());
        let s = normalize(&adapter(), &p).unwrap();
        assert_eq!(s.nonce, 7);
        assert_eq!(s.distance_cm, 25_000);
        assert_eq!(s.speed_cm_s, 800);
    }
}
