use crate::bcn::bc1::decode_bc1_block;

#[inline]
pub fn decode_bc3_alpha(data: &[u8], outbuf: &mut [u32], channel: usize) {
    // u16 avoids overflow; interpolated values round to nearest, matching Unity's own decoder
    let mut a: [u16; 8] = [data[0] as u16, data[1] as u16, 0, 0, 0, 0, 0, 0];
    if a[0] > a[1] {
        a[2] = (a[0] * 6 + a[1] + 3) / 7;
        a[3] = (a[0] * 5 + a[1] * 2 + 3) / 7;
        a[4] = (a[0] * 4 + a[1] * 3 + 3) / 7;
        a[5] = (a[0] * 3 + a[1] * 4 + 3) / 7;
        a[6] = (a[0] * 2 + a[1] * 5 + 3) / 7;
        a[7] = (a[0] + a[1] * 6 + 3) / 7;
    } else {
        a[2] = (a[0] * 4 + a[1] + 2) / 5;
        a[3] = (a[0] * 3 + a[1] * 2 + 2) / 5;
        a[4] = (a[0] * 2 + a[1] * 3 + 2) / 5;
        a[5] = (a[0] + a[1] * 4 + 2) / 5;
        a[6] = 0;
        a[7] = 255;
    }

    let mut d: usize = (u64::from_le_bytes(data[..8].try_into().unwrap()) >> 16) as usize;

    let channel_shift = channel * 8;
    let channel_mask = 0xFFFFFFFF ^ (0xFF << channel_shift);
    outbuf.iter_mut().for_each(|p| {
        *p = (*p & channel_mask) | (a[d & 7] as u32) << channel_shift;
        d >>= 3;
    });
}

#[inline]
pub fn decode_bc3_block(data: &[u8], outbuf: &mut [u32]) {
    decode_bc1_block(&data[8..], outbuf);
    decode_bc3_alpha(data, outbuf, 3);
}

#[cfg(test)]
mod tests {
    use super::decode_bc3_alpha;

    fn palette(a0: u8, a1: u8) -> [u8; 8] {
        // indices 0..8 for the first 8 pixels, packed as 3 bits each
        let indices: u64 = (0..8).map(|i| (i as u64) << (3 * i)).sum();
        let mut block = [a0, a1, 0, 0, 0, 0, 0, 0];
        block[2..8].copy_from_slice(&indices.to_le_bytes()[..6]);
        let mut out = [0u32; 16];
        decode_bc3_alpha(&block, &mut out, 3);
        core::array::from_fn(|i| (out[i] >> 24) as u8)
    }

    #[test]
    fn interpolated_alpha_rounds_to_nearest() {
        assert_eq!(palette(255, 0), [255, 0, 219, 182, 146, 109, 73, 36]);
        assert_eq!(palette(0, 4), [0, 4, 1, 2, 2, 3, 0, 255]);
    }
}
