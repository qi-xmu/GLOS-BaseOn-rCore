/// Convert to intel byte order
#[allow(unused)]
pub fn bytes_order_u16(x: u16) -> u16 {
    let mut y: [u8; 2];
    y = x.to_be_bytes();
    y.reverse();
    ((y[0] as u16) << 8) | y[1] as u16
}

// 转化为正确的字节序
#[allow(unused)]
pub fn bytes_order_u32(x: u32) -> u32 {
    let mut y: [u8; 4];
    y = x.to_be_bytes();
    y.reverse();
    ((y[0] as u32) << 24) | ((y[1] as u32) << 16) | ((y[2] as u32) << 8) | y[3] as u32
}

pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}
