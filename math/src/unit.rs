pub trait Unit {}

pub trait LengthUnit: Unit {}
pub trait AngleUnit: Unit {}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Hash, Eq, PartialEq)]
pub struct UnitScalar<T, U: LengthUnit> {
  value: T,
  phantom: PhantomData<U>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Hash, Eq, PartialEq)]
pub struct UnitAngle<T, U: AngleUnit> {
  value: T,
  phantom: PhantomData<U>,
}

pub struct RadUnit {}
impl Unit for RadUnit {}
impl AngleUnit for RadUnit {}

pub struct DegUnit {}
impl Unit for DegUnit {}
impl AngleUnit for DegUnit {}

pub type Rad<T> = UnitAngle<T, RadUnit>;
pub type Deg<T> = UnitAngle<T, DegUnit>;