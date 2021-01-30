use rendiation_math::Scalar;
use rendiation_ral::AddressMode;

/// How edges should be handled in texture addressing.
pub trait TextureAddressMode {
  /// correct uv to [1, 1]
  fn correct<T: Scalar>(uv: T) -> T;
}

pub trait RALAddressMode {
  const ENUM: AddressMode;
}

/// Clamp the value to the edge of the texture
///
/// -0.25 -> 0.0
/// 1.25  -> 1.0
pub struct ClampToEdge;
impl RALAddressMode for ClampToEdge {
  const ENUM: AddressMode = AddressMode::ClampToEdge;
}
impl TextureAddressMode for ClampToEdge {
  fn correct<T: Scalar>(uv: T) -> T {
    uv.max(T::zero()).min(T::one())
  }
}

#[test]
fn clamp() {
  assert_eq!(ClampToEdge::correct(1.25), 1.0);
  assert_eq!(ClampToEdge::correct(-0.25), 0.0);
}

/// Repeat the texture in a tiling fashion
///
/// -0.25 -> 0.75
/// 1.25 -> 0.25
pub struct Repeat;
impl RALAddressMode for Repeat {
  const ENUM: AddressMode = AddressMode::Repeat;
}
impl TextureAddressMode for Repeat {
  fn correct<T: Scalar>(uv: T) -> T {
    uv % T::one()
  }
}

#[test]
fn repeat() {
  assert_eq!(Repeat::correct(1.25), 0.25);
  assert_eq!(Repeat::correct(-0.25), 0.75);
}

/// Repeat the texture, mirroring it every repeat
///
/// -0.25 -> 0.25
/// 1.25 -> 0.75
pub struct MirrorRepeat;
impl RALAddressMode for MirrorRepeat {
  const ENUM: AddressMode = AddressMode::MirrorRepeat;
}
impl TextureAddressMode for MirrorRepeat {
  fn correct<T: Scalar>(uv: T) -> T {
    todo!()
  }
}

#[test]
fn mirror_repeat() {
  assert_eq!(MirrorRepeat::correct(1.25), 0.75);
  assert_eq!(MirrorRepeat::correct(-0.25), 0.25);
}
