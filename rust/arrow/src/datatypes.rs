// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Defines the data-types of Arrow arrays.
//!
//! For an overview of the terminology used within the arrow project and more general
//! information regarding data-types and memory layouts see
//! [here](https://arrow.apache.org/docs/memory_layout.html).

use std::collections::HashMap;
use std::fmt;
use std::mem::size_of;
#[cfg(feature = "simd")]
use std::ops::{Add, Div, Mul, Sub};
use std::slice::from_raw_parts;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(feature = "simd")]
use packed_simd::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::{
    json, Number, Value, Value::Number as VNumber, Value::String as VString,
};

use crate::error::{ArrowError, Result};

/// The possible relative types that are supported.
///
/// The variants of this enum include primitive fixed size types as well as parametric or
/// nested types.
/// Currently the Rust implementation supports the following  nested types:
///  - `List<T>`
///  - `Struct<T, U, V, ...>`
///
/// Nested types can themselves be nested within other arrays.
/// For more information on these types please see
/// [here](https://arrow.apache.org/docs/memory_layout.html).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float16,
    Float32,
    Float64,
    /// A timestamp with an optional timezone
    Timestamp(TimeUnit, Option<Arc<String>>),
    Date32(DateUnit),
    Date64(DateUnit),
    Time32(TimeUnit),
    Time64(TimeUnit),
    Duration(TimeUnit),
    Interval(IntervalUnit),
    Binary,
    FixedSizeBinary(i32),
    Utf8,
    List(Box<DataType>),
    FixedSizeList(Box<DataType>, i32),
    Struct(Vec<Field>),
    Dictionary(Box<DataType>, Box<DataType>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DateUnit {
    Day,
    Millisecond,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeUnit {
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IntervalUnit {
    YearMonth,
    DayTime,
}

/// Contains the meta-data for a single relative type.
///
/// The `Schema` object is an ordered collection of `Field` objects.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Field {
    name: String,
    data_type: DataType,
    nullable: bool,
    dict_id: i64,
    dict_is_ordered: bool,
}

pub trait ArrowNativeType:
    fmt::Debug + Send + Sync + Copy + PartialOrd + FromStr + 'static
{
    fn into_json_value(self) -> Option<Value>;

    /// Convert native type from usize.
    fn from_usize(_: usize) -> Option<Self> {
        None
    }

    /// Convert native type to usize.
    fn to_usize(&self) -> Option<usize> {
        None
    }
}

/// Trait indicating a primitive fixed-width type (bool, ints and floats).
pub trait ArrowPrimitiveType: 'static {
    /// Corresponding Rust native type for the primitive type.
    type Native: ArrowNativeType;

    /// Returns the corresponding Arrow data type of this primitive type.
    fn get_data_type() -> DataType;

    /// Returns the bit width of this primitive type.
    fn get_bit_width() -> usize;

    /// Returns a default value of this primitive type.
    ///
    /// This is useful for aggregate array ops like `sum()`, `mean()`.
    fn default_value() -> Self::Native;
}

impl ArrowNativeType for bool {
    fn into_json_value(self) -> Option<Value> {
        Some(self.into())
    }
}

impl ArrowNativeType for i8 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for i16 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for i32 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for i64 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for u8 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for u16 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for u32 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for u64 {
    fn into_json_value(self) -> Option<Value> {
        Some(VNumber(Number::from(self)))
    }

    fn from_usize(v: usize) -> Option<Self> {
        num::FromPrimitive::from_usize(v)
    }

    fn to_usize(&self) -> Option<usize> {
        num::ToPrimitive::to_usize(self)
    }
}

impl ArrowNativeType for f32 {
    fn into_json_value(self) -> Option<Value> {
        Number::from_f64(f64::round(self as f64 * 1000.0) / 1000.0)
            .map(|num| VNumber(num))
    }
}

impl ArrowNativeType for f64 {
    fn into_json_value(self) -> Option<Value> {
        Number::from_f64(self).map(|num| VNumber(num))
    }
}

macro_rules! make_type {
    ($name:ident, $native_ty:ty, $data_ty:expr, $bit_width:expr, $default_val:expr) => {
        pub struct $name {}

        impl ArrowPrimitiveType for $name {
            type Native = $native_ty;

            fn get_data_type() -> DataType {
                $data_ty
            }

            fn get_bit_width() -> usize {
                $bit_width
            }

            fn default_value() -> Self::Native {
                $default_val
            }
        }
    };
}

make_type!(BooleanType, bool, DataType::Boolean, 1, false);
make_type!(Int8Type, i8, DataType::Int8, 8, 0i8);
make_type!(Int16Type, i16, DataType::Int16, 16, 0i16);
make_type!(Int32Type, i32, DataType::Int32, 32, 0i32);
make_type!(Int64Type, i64, DataType::Int64, 64, 0i64);
make_type!(UInt8Type, u8, DataType::UInt8, 8, 0u8);
make_type!(UInt16Type, u16, DataType::UInt16, 16, 0u16);
make_type!(UInt32Type, u32, DataType::UInt32, 32, 0u32);
make_type!(UInt64Type, u64, DataType::UInt64, 64, 0u64);
make_type!(Float32Type, f32, DataType::Float32, 32, 0.0f32);
make_type!(Float64Type, f64, DataType::Float64, 64, 0.0f64);
make_type!(
    TimestampSecondType,
    i64,
    DataType::Timestamp(TimeUnit::Second, None),
    64,
    0i64
);
make_type!(
    TimestampMillisecondType,
    i64,
    DataType::Timestamp(TimeUnit::Millisecond, None),
    64,
    0i64
);
make_type!(
    TimestampMicrosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Microsecond, None),
    64,
    0i64
);
make_type!(
    TimestampNanosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Nanosecond, None),
    64,
    0i64
);
make_type!(Date32Type, i32, DataType::Date32(DateUnit::Day), 32, 0i32);
make_type!(
    Date64Type,
    i64,
    DataType::Date64(DateUnit::Millisecond),
    64,
    0i64
);
make_type!(
    Time32SecondType,
    i32,
    DataType::Time32(TimeUnit::Second),
    32,
    0i32
);
make_type!(
    Time32MillisecondType,
    i32,
    DataType::Time32(TimeUnit::Millisecond),
    32,
    0i32
);
make_type!(
    Time64MicrosecondType,
    i64,
    DataType::Time64(TimeUnit::Microsecond),
    64,
    0i64
);
make_type!(
    Time64NanosecondType,
    i64,
    DataType::Time64(TimeUnit::Nanosecond),
    64,
    0i64
);
make_type!(
    IntervalYearMonthType,
    i32,
    DataType::Interval(IntervalUnit::YearMonth),
    32,
    0i32
);
make_type!(
    IntervalDayTimeType,
    i64,
    DataType::Interval(IntervalUnit::DayTime),
    64,
    0i64
);
make_type!(
    DurationSecondType,
    i64,
    DataType::Duration(TimeUnit::Second),
    64,
    0i64
);
make_type!(
    DurationMillisecondType,
    i64,
    DataType::Duration(TimeUnit::Millisecond),
    64,
    0i64
);
make_type!(
    DurationMicrosecondType,
    i64,
    DataType::Duration(TimeUnit::Microsecond),
    64,
    0i64
);
make_type!(
    DurationNanosecondType,
    i64,
    DataType::Duration(TimeUnit::Nanosecond),
    64,
    0i64
);

/// A subtype of primitive type that represents legal dictionary keys.
/// See https://arrow.apache.org/docs/format/Columnar.html
pub trait ArrowDictionaryKeyType: ArrowPrimitiveType {}

impl ArrowDictionaryKeyType for Int8Type {}

impl ArrowDictionaryKeyType for Int16Type {}

impl ArrowDictionaryKeyType for Int32Type {}

impl ArrowDictionaryKeyType for Int64Type {}

/// A subtype of primitive type that represents numeric values.
///
/// SIMD operations are defined in this trait if available on the target system.
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
pub trait ArrowNumericType: ArrowPrimitiveType
where
    Self::Simd: Add<Output = Self::Simd>
        + Sub<Output = Self::Simd>
        + Mul<Output = Self::Simd>
        + Div<Output = Self::Simd>,
{
    /// Defines the SIMD type that should be used for this numeric type
    type Simd;

    /// Defines the SIMD Mask type that should be used for this numeric type
    type SimdMask;

    /// The number of SIMD lanes available
    fn lanes() -> usize;

    /// Initializes a SIMD register to a constant value
    fn init(value: Self::Native) -> Self::Simd;

    /// Loads a slice into a SIMD register
    fn load(slice: &[Self::Native]) -> Self::Simd;

    /// Creates a new SIMD mask for this SIMD type filling it with `value`
    fn mask_init(value: bool) -> Self::SimdMask;

    /// Gets the value of a single lane in a SIMD mask
    fn mask_get(mask: &Self::SimdMask, idx: usize) -> bool;

    /// Sets the value of a single lane of a SIMD mask
    fn mask_set(mask: Self::SimdMask, idx: usize, value: bool) -> Self::SimdMask;

    /// Selects elements of `a` and `b` using `mask`
    fn mask_select(mask: Self::SimdMask, a: Self::Simd, b: Self::Simd) -> Self::Simd;

    /// Returns `true` if any of the lanes in the mask are `true`
    fn mask_any(mask: Self::SimdMask) -> bool;

    /// Performs a SIMD binary operation
    fn bin_op<F: Fn(Self::Simd, Self::Simd) -> Self::Simd>(
        left: Self::Simd,
        right: Self::Simd,
        op: F,
    ) -> Self::Simd;

    // SIMD version of equal
    fn eq(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    // SIMD version of not equal
    fn ne(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    // SIMD version of less than
    fn lt(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    // SIMD version of less than or equal to
    fn le(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    // SIMD version of greater than
    fn gt(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    // SIMD version of greater than or equal to
    fn ge(left: Self::Simd, right: Self::Simd) -> Self::SimdMask;

    /// Writes a SIMD result back to a slice
    fn write(simd_result: Self::Simd, slice: &mut [Self::Native]);
}

#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(feature = "simd")
))]
pub trait ArrowNumericType: ArrowPrimitiveType {}

macro_rules! make_numeric_type {
    ($impl_ty:ty, $native_ty:ty, $simd_ty:ident, $simd_mask_ty:ident) => {
        #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
        impl ArrowNumericType for $impl_ty {
            type Simd = $simd_ty;

            type SimdMask = $simd_mask_ty;

            fn lanes() -> usize {
                Self::Simd::lanes()
            }

            fn init(value: Self::Native) -> Self::Simd {
                Self::Simd::splat(value)
            }

            fn load(slice: &[Self::Native]) -> Self::Simd {
                unsafe { Self::Simd::from_slice_unaligned_unchecked(slice) }
            }

            fn mask_init(value: bool) -> Self::SimdMask {
                Self::SimdMask::splat(value)
            }

            fn mask_get(mask: &Self::SimdMask, idx: usize) -> bool {
                unsafe { mask.extract_unchecked(idx) }
            }

            fn mask_set(mask: Self::SimdMask, idx: usize, value: bool) -> Self::SimdMask {
                unsafe { mask.replace_unchecked(idx, value) }
            }

            /// Selects elements of `a` and `b` using `mask`
            fn mask_select(
                mask: Self::SimdMask,
                a: Self::Simd,
                b: Self::Simd,
            ) -> Self::Simd {
                mask.select(a, b)
            }

            fn mask_any(mask: Self::SimdMask) -> bool {
                mask.any()
            }

            fn bin_op<F: Fn(Self::Simd, Self::Simd) -> Self::Simd>(
                left: Self::Simd,
                right: Self::Simd,
                op: F,
            ) -> Self::Simd {
                op(left, right)
            }

            fn eq(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.eq(right)
            }

            fn ne(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.ne(right)
            }

            fn lt(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.lt(right)
            }

            fn le(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.le(right)
            }

            fn gt(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.gt(right)
            }

            fn ge(left: Self::Simd, right: Self::Simd) -> Self::SimdMask {
                left.ge(right)
            }

            fn write(simd_result: Self::Simd, slice: &mut [Self::Native]) {
                unsafe { simd_result.write_to_slice_unaligned_unchecked(slice) };
            }
        }
        #[cfg(any(
            not(any(target_arch = "x86", target_arch = "x86_64")),
            not(feature = "simd")
        ))]
        impl ArrowNumericType for $impl_ty {}
    };
}

make_numeric_type!(Int8Type, i8, i8x64, m8x64);
make_numeric_type!(Int16Type, i16, i16x32, m16x32);
make_numeric_type!(Int32Type, i32, i32x16, m32x16);
make_numeric_type!(Int64Type, i64, i64x8, m64x8);
make_numeric_type!(UInt8Type, u8, u8x64, m8x64);
make_numeric_type!(UInt16Type, u16, u16x32, m16x32);
make_numeric_type!(UInt32Type, u32, u32x16, m32x16);
make_numeric_type!(UInt64Type, u64, u64x8, m64x8);
make_numeric_type!(Float32Type, f32, f32x16, m32x16);
make_numeric_type!(Float64Type, f64, f64x8, m64x8);

make_numeric_type!(TimestampSecondType, i64, i64x8, m64x8);
make_numeric_type!(TimestampMillisecondType, i64, i64x8, m64x8);
make_numeric_type!(TimestampMicrosecondType, i64, i64x8, m64x8);
make_numeric_type!(TimestampNanosecondType, i64, i64x8, m64x8);
make_numeric_type!(Date32Type, i32, i32x16, m32x16);
make_numeric_type!(Date64Type, i64, i64x8, m64x8);
make_numeric_type!(Time32SecondType, i32, i32x16, m32x16);
make_numeric_type!(Time32MillisecondType, i32, i32x16, m32x16);
make_numeric_type!(Time64MicrosecondType, i64, i64x8, m64x8);
make_numeric_type!(Time64NanosecondType, i64, i64x8, m64x8);
make_numeric_type!(IntervalYearMonthType, i32, i32x16, m32x16);
make_numeric_type!(IntervalDayTimeType, i64, i64x8, m64x8);
make_numeric_type!(DurationSecondType, i64, i64x8, m64x8);
make_numeric_type!(DurationMillisecondType, i64, i64x8, m64x8);
make_numeric_type!(DurationMicrosecondType, i64, i64x8, m64x8);
make_numeric_type!(DurationNanosecondType, i64, i64x8, m64x8);

/// A subtype of primitive type that represents temporal values.
pub trait ArrowTemporalType: ArrowPrimitiveType {}

impl ArrowTemporalType for TimestampSecondType {}
impl ArrowTemporalType for TimestampMillisecondType {}
impl ArrowTemporalType for TimestampMicrosecondType {}
impl ArrowTemporalType for TimestampNanosecondType {}
impl ArrowTemporalType for Date32Type {}
impl ArrowTemporalType for Date64Type {}
impl ArrowTemporalType for Time32SecondType {}
impl ArrowTemporalType for Time32MillisecondType {}
impl ArrowTemporalType for Time64MicrosecondType {}
impl ArrowTemporalType for Time64NanosecondType {}
// impl ArrowTemporalType for IntervalYearMonthType {}
// impl ArrowTemporalType for IntervalDayTimeType {}

/// A timestamp type allows us to create array builders that take a timestamp
pub trait ArrowTimestampType: ArrowTemporalType {
    fn get_time_unit() -> TimeUnit;
}

impl ArrowTimestampType for TimestampSecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Second
    }
}
impl ArrowTimestampType for TimestampMillisecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Millisecond
    }
}
impl ArrowTimestampType for TimestampMicrosecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Microsecond
    }
}
impl ArrowTimestampType for TimestampNanosecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Nanosecond
    }
}

/// Allows conversion from supported Arrow types to a byte slice.
pub trait ToByteSlice {
    /// Converts this instance into a byte slice
    fn to_byte_slice(&self) -> &[u8];
}

impl<T: ArrowNativeType> ToByteSlice for [T] {
    fn to_byte_slice(&self) -> &[u8] {
        let raw_ptr = self.as_ptr() as *const T as *const u8;
        unsafe { from_raw_parts(raw_ptr, self.len() * size_of::<T>()) }
    }
}

impl<T: ArrowNativeType> ToByteSlice for T {
    fn to_byte_slice(&self) -> &[u8] {
        let raw_ptr = self as *const T as *const u8;
        unsafe { from_raw_parts(raw_ptr, size_of::<T>()) }
    }
}

impl DataType {
    /// Parse a data type from a JSON representation
    fn from(json: &Value) -> Result<DataType> {
        match *json {
            Value::Object(ref map) => match map.get("name") {
                Some(s) if s == "bool" => Ok(DataType::Boolean),
                Some(s) if s == "binary" => Ok(DataType::Binary),
                Some(s) if s == "utf8" => Ok(DataType::Utf8),
                Some(s) if s == "fixedsizebinary" => {
                    // return a list with any type as its child isn't defined in the map
                    if let Some(Value::Number(size)) = map.get("byteWidth") {
                        Ok(DataType::FixedSizeBinary(size.as_i64().unwrap() as i32))
                    } else {
                        Err(ArrowError::ParseError(format!(
                            "Expecting a byteWidth for fixedsizebinary",
                        )))
                    }
                }
                Some(s) if s == "floatingpoint" => match map.get("precision") {
                    Some(p) if p == "HALF" => Ok(DataType::Float16),
                    Some(p) if p == "SINGLE" => Ok(DataType::Float32),
                    Some(p) if p == "DOUBLE" => Ok(DataType::Float64),
                    _ => Err(ArrowError::ParseError(
                        "floatingpoint precision missing or invalid".to_string(),
                    )),
                },
                Some(s) if s == "timestamp" => {
                    let unit = match map.get("unit") {
                        Some(p) if p == "SECOND" => Ok(TimeUnit::Second),
                        Some(p) if p == "MILLISECOND" => Ok(TimeUnit::Millisecond),
                        Some(p) if p == "MICROSECOND" => Ok(TimeUnit::Microsecond),
                        Some(p) if p == "NANOSECOND" => Ok(TimeUnit::Nanosecond),
                        _ => Err(ArrowError::ParseError(
                            "timestamp unit missing or invalid".to_string(),
                        )),
                    };
                    let tz = match map.get("timezone") {
                        None => Ok(None),
                        Some(VString(tz)) => Ok(Some(Arc::new(tz.to_string()))),
                        _ => Err(ArrowError::ParseError(
                            "timezone must be a string".to_string(),
                        )),
                    };
                    Ok(DataType::Timestamp(unit?, tz?))
                }
                Some(s) if s == "date" => match map.get("unit") {
                    Some(p) if p == "DAY" => Ok(DataType::Date32(DateUnit::Day)),
                    Some(p) if p == "MILLISECOND" => {
                        Ok(DataType::Date64(DateUnit::Millisecond))
                    }
                    _ => Err(ArrowError::ParseError(
                        "date unit missing or invalid".to_string(),
                    )),
                },
                Some(s) if s == "time" => {
                    let unit = match map.get("unit") {
                        Some(p) if p == "SECOND" => Ok(TimeUnit::Second),
                        Some(p) if p == "MILLISECOND" => Ok(TimeUnit::Millisecond),
                        Some(p) if p == "MICROSECOND" => Ok(TimeUnit::Microsecond),
                        Some(p) if p == "NANOSECOND" => Ok(TimeUnit::Nanosecond),
                        _ => Err(ArrowError::ParseError(
                            "time unit missing or invalid".to_string(),
                        )),
                    };
                    match map.get("bitWidth") {
                        Some(p) if p == 32 => Ok(DataType::Time32(unit?)),
                        Some(p) if p == 64 => Ok(DataType::Time64(unit?)),
                        _ => Err(ArrowError::ParseError(
                            "time bitWidth missing or invalid".to_string(),
                        )),
                    }
                }
                Some(s) if s == "duration" => match map.get("unit") {
                    Some(p) if p == "SECOND" => Ok(DataType::Duration(TimeUnit::Second)),
                    Some(p) if p == "MILLISECOND" => {
                        Ok(DataType::Duration(TimeUnit::Millisecond))
                    }
                    Some(p) if p == "MICROSECOND" => {
                        Ok(DataType::Duration(TimeUnit::Microsecond))
                    }
                    Some(p) if p == "NANOSECOND" => {
                        Ok(DataType::Duration(TimeUnit::Nanosecond))
                    }
                    _ => Err(ArrowError::ParseError(
                        "time unit missing or invalid".to_string(),
                    )),
                },
                Some(s) if s == "interval" => match map.get("unit") {
                    Some(p) if p == "DAY_TIME" => {
                        Ok(DataType::Interval(IntervalUnit::DayTime))
                    }
                    Some(p) if p == "YEAR_MONTH" => {
                        Ok(DataType::Interval(IntervalUnit::YearMonth))
                    }
                    _ => Err(ArrowError::ParseError(
                        "interval unit missing or invalid".to_string(),
                    )),
                },
                Some(s) if s == "int" => match map.get("isSigned") {
                    Some(&Value::Bool(true)) => match map.get("bitWidth") {
                        Some(&Value::Number(ref n)) => match n.as_u64() {
                            Some(8) => Ok(DataType::Int8),
                            Some(16) => Ok(DataType::Int16),
                            Some(32) => Ok(DataType::Int32),
                            Some(64) => Ok(DataType::Int32),
                            _ => Err(ArrowError::ParseError(
                                "int bitWidth missing or invalid".to_string(),
                            )),
                        },
                        _ => Err(ArrowError::ParseError(
                            "int bitWidth missing or invalid".to_string(),
                        )),
                    },
                    Some(&Value::Bool(false)) => match map.get("bitWidth") {
                        Some(&Value::Number(ref n)) => match n.as_u64() {
                            Some(8) => Ok(DataType::UInt8),
                            Some(16) => Ok(DataType::UInt16),
                            Some(32) => Ok(DataType::UInt32),
                            Some(64) => Ok(DataType::UInt64),
                            _ => Err(ArrowError::ParseError(
                                "int bitWidth missing or invalid".to_string(),
                            )),
                        },
                        _ => Err(ArrowError::ParseError(
                            "int bitWidth missing or invalid".to_string(),
                        )),
                    },
                    _ => Err(ArrowError::ParseError(
                        "int signed missing or invalid".to_string(),
                    )),
                },
                Some(s) if s == "list" => {
                    // return a list with any type as its child isn't defined in the map
                    Ok(DataType::List(Box::new(DataType::Boolean)))
                }
                Some(s) if s == "fixedsizelist" => {
                    // return a list with any type as its child isn't defined in the map
                    if let Some(Value::Number(size)) = map.get("listSize") {
                        Ok(DataType::FixedSizeList(
                            Box::new(DataType::Boolean),
                            size.as_i64().unwrap() as i32,
                        ))
                    } else {
                        Err(ArrowError::ParseError(format!(
                            "Expecting a listSize for fixedsizelist",
                        )))
                    }
                }
                Some(s) if s == "struct" => {
                    // return an empty `struct` type as its children aren't defined in the map
                    Ok(DataType::Struct(vec![]))
                }
                Some(other) => Err(ArrowError::ParseError(format!(
                    "invalid or unsupported type name: {} in {:?}",
                    other, json
                ))),
                None => Err(ArrowError::ParseError("type name missing".to_string())),
            },
            _ => Err(ArrowError::ParseError(
                "invalid json value type".to_string(),
            )),
        }
    }

    /// Generate a JSON representation of the data type
    pub fn to_json(&self) -> Value {
        match self {
            DataType::Boolean => json!({"name": "bool"}),
            DataType::Int8 => json!({"name": "int", "bitWidth": 8, "isSigned": true}),
            DataType::Int16 => json!({"name": "int", "bitWidth": 16, "isSigned": true}),
            DataType::Int32 => json!({"name": "int", "bitWidth": 32, "isSigned": true}),
            DataType::Int64 => json!({"name": "int", "bitWidth": 64, "isSigned": true}),
            DataType::UInt8 => json!({"name": "int", "bitWidth": 8, "isSigned": false}),
            DataType::UInt16 => json!({"name": "int", "bitWidth": 16, "isSigned": false}),
            DataType::UInt32 => json!({"name": "int", "bitWidth": 32, "isSigned": false}),
            DataType::UInt64 => json!({"name": "int", "bitWidth": 64, "isSigned": false}),
            DataType::Float16 => json!({"name": "floatingpoint", "precision": "HALF"}),
            DataType::Float32 => json!({"name": "floatingpoint", "precision": "SINGLE"}),
            DataType::Float64 => json!({"name": "floatingpoint", "precision": "DOUBLE"}),
            DataType::Utf8 => json!({"name": "utf8"}),
            DataType::Binary => json!({"name": "binary"}),
            DataType::FixedSizeBinary(byte_width) => {
                json!({"name": "fixedsizebinary", "byteWidth": byte_width})
            }
            DataType::Struct(_) => json!({"name": "struct"}),
            DataType::List(_) => json!({ "name": "list"}),
            DataType::FixedSizeList(_, length) => {
                json!({"name":"fixedsizelist", "listSize": length})
            }
            DataType::Time32(unit) => {
                json!({"name": "time", "bitWidth": 32, "unit": match unit {
                    TimeUnit::Second => "SECOND",
                    TimeUnit::Millisecond => "MILLISECOND",
                    TimeUnit::Microsecond => "MICROSECOND",
                    TimeUnit::Nanosecond => "NANOSECOND",
                }})
            }
            DataType::Time64(unit) => {
                json!({"name": "time", "bitWidth": 64, "unit": match unit {
                    TimeUnit::Second => "SECOND",
                    TimeUnit::Millisecond => "MILLISECOND",
                    TimeUnit::Microsecond => "MICROSECOND",
                    TimeUnit::Nanosecond => "NANOSECOND",
                }})
            }
            DataType::Date32(unit) | DataType::Date64(unit) => {
                json!({"name": "date", "unit": match unit {
                    DateUnit::Day => "DAY",
                    DateUnit::Millisecond => "MILLISECOND",
                }})
            }
            DataType::Timestamp(unit, None) => {
                json!({"name": "timestamp", "unit": match unit {
                    TimeUnit::Second => "SECOND",
                    TimeUnit::Millisecond => "MILLISECOND",
                    TimeUnit::Microsecond => "MICROSECOND",
                    TimeUnit::Nanosecond => "NANOSECOND",
                }})
            }
            DataType::Timestamp(unit, Some(tz)) => {
                json!({"name": "timestamp", "unit": match unit {
                    TimeUnit::Second => "SECOND",
                    TimeUnit::Millisecond => "MILLISECOND",
                    TimeUnit::Microsecond => "MICROSECOND",
                    TimeUnit::Nanosecond => "NANOSECOND",
                }, "timezone": tz})
            }
            DataType::Interval(unit) => json!({"name": "interval", "unit": match unit {
                IntervalUnit::YearMonth => "YEAR_MONTH",
                IntervalUnit::DayTime => "DAY_TIME",
            }}),
            DataType::Duration(unit) => json!({"name": "duration", "unit": match unit {
                TimeUnit::Second => "SECOND",
                TimeUnit::Millisecond => "MILLISECOND",
                TimeUnit::Microsecond => "MICROSECOND",
                TimeUnit::Nanosecond => "NANOSECOND",
            }}),
            DataType::Dictionary(_, _) => json!({ "name": "dictionary"}),
        }
    }
}

impl Field {
    /// Creates a new field
    pub fn new(name: &str, data_type: DataType, nullable: bool) -> Self {
        Field {
            name: name.to_string(),
            data_type,
            nullable,
            dict_id: 0,
            dict_is_ordered: false,
        }
    }

    /// Creates a new field
    pub fn new_dict(
        name: &str,
        data_type: DataType,
        nullable: bool,
        dict_id: i64,
        dict_is_ordered: bool,
    ) -> Self {
        Field {
            name: name.to_string(),
            data_type,
            nullable,
            dict_id,
            dict_is_ordered,
        }
    }

    /// Returns an immutable reference to the `Field`'s name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns an immutable reference to the `Field`'s  data-type
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Indicates whether this `Field` supports null values
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Parse a `Field` definition from a JSON representation
    pub fn from(json: &Value) -> Result<Self> {
        match *json {
            Value::Object(ref map) => {
                let name = match map.get("name") {
                    Some(&Value::String(ref name)) => name.to_string(),
                    _ => {
                        return Err(ArrowError::ParseError(
                            "Field missing 'name' attribute".to_string(),
                        ));
                    }
                };
                let nullable = match map.get("nullable") {
                    Some(&Value::Bool(b)) => b,
                    _ => {
                        return Err(ArrowError::ParseError(
                            "Field missing 'nullable' attribute".to_string(),
                        ));
                    }
                };
                let data_type = match map.get("type") {
                    Some(t) => DataType::from(t)?,
                    _ => {
                        return Err(ArrowError::ParseError(
                            "Field missing 'type' attribute".to_string(),
                        ));
                    }
                };
                // if data_type is a struct or list, get its children
                let data_type = match data_type {
                    DataType::List(_) | DataType::FixedSizeList(_, _) => {
                        match map.get("children") {
                            Some(Value::Array(values)) => {
                                if values.len() != 1 {
                                    return Err(ArrowError::ParseError(
                                    "Field 'children' must have one element for a list data type".to_string(),
                                ));
                                }
                                match data_type {
                                    DataType::List(_) => DataType::List(Box::new(
                                        Self::from(&values[0])?.data_type,
                                    )),
                                    DataType::FixedSizeList(_, int) => {
                                        DataType::FixedSizeList(
                                            Box::new(Self::from(&values[0])?.data_type),
                                            int,
                                        )
                                    }
                                    _ => unreachable!(
                                        "Data type should be a list or fixedsizelist"
                                    ),
                                }
                            }
                            Some(_) => {
                                return Err(ArrowError::ParseError(
                                    "Field 'children' must be an array".to_string(),
                                ))
                            }
                            None => {
                                return Err(ArrowError::ParseError(
                                    "Field missing 'children' attribute".to_string(),
                                ));
                            }
                        }
                    }
                    DataType::Struct(mut fields) => match map.get("children") {
                        Some(Value::Array(values)) => {
                            let struct_fields: Result<Vec<Field>> =
                                values.iter().map(|v| Field::from(v)).collect();
                            fields.append(&mut struct_fields?);
                            DataType::Struct(fields)
                        }
                        Some(_) => {
                            return Err(ArrowError::ParseError(
                                "Field 'children' must be an array".to_string(),
                            ))
                        }
                        None => {
                            return Err(ArrowError::ParseError(
                                "Field missing 'children' attribute".to_string(),
                            ));
                        }
                    },
                    _ => data_type,
                };

                let mut dict_id = 0;
                let mut dict_is_ordered = false;

                let data_type = match map.get("dictionary") {
                    Some(dictionary) => {
                        let index_type = match dictionary.get("indexType") {
                            Some(t) => DataType::from(t)?,
                            _ => {
                                return Err(ArrowError::ParseError(
                                    "Field missing 'indexType' attribute".to_string(),
                                ));
                            }
                        };
                        dict_id = match dictionary.get("id") {
                            Some(Value::Number(n)) => n.as_i64().unwrap(),
                            _ => {
                                return Err(ArrowError::ParseError(
                                    "Field missing 'id' attribute".to_string(),
                                ));
                            }
                        };
                        dict_is_ordered = match dictionary.get("isOrdered") {
                            Some(&Value::Bool(n)) => n,
                            _ => {
                                return Err(ArrowError::ParseError(
                                    "Field missing 'isOrdered' attribute".to_string(),
                                ));
                            }
                        };
                        DataType::Dictionary(Box::new(index_type), Box::new(data_type))
                    }
                    _ => data_type,
                };
                Ok(Field {
                    name,
                    nullable,
                    data_type,
                    dict_id,
                    dict_is_ordered,
                })
            }
            _ => Err(ArrowError::ParseError(
                "Invalid json value type for field".to_string(),
            )),
        }
    }

    /// Generate a JSON representation of the `Field`
    pub fn to_json(&self) -> Value {
        let children: Vec<Value> = match self.data_type() {
            DataType::Struct(fields) => fields.iter().map(|f| f.to_json()).collect(),
            DataType::List(dtype) => {
                let item = Field::new("item", *dtype.clone(), self.nullable);
                vec![item.to_json()]
            }
            DataType::FixedSizeList(dtype, _) => {
                let item = Field::new("item", *dtype.clone(), self.nullable);
                vec![item.to_json()]
            }
            _ => vec![],
        };
        match self.data_type() {
            DataType::Dictionary(ref index_type, ref value_type) => json!({
                "name": self.name,
                "nullable": self.nullable,
                "type": value_type.to_json(),
                "children": children,
                "dictionary": {
                    "id": self.dict_id,
                    "indexType": index_type.to_json(),
                    "isOrdered": self.dict_is_ordered
                }
            }),
            _ => json!({
                "name": self.name,
                "nullable": self.nullable,
                "type": self.data_type.to_json(),
                "children": children
            }),
        }
    }

    /// Converts to a `String` representation of the `Field`
    pub fn to_string(&self) -> String {
        format!("{}: {:?}", self.name, self.data_type)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Describes the meta-data of an ordered sequence of relative types.
///
/// Note that this information is only part of the meta-data and not part of the physical
/// memory layout.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub(crate) fields: Vec<Field>,
    /// A map of key-value pairs containing additional meta data.
    #[serde(default)]
    pub(crate) metadata: HashMap<String, String>,
}

impl Schema {
    /// Creates an empty `Schema`
    pub fn empty() -> Self {
        Self {
            fields: vec![],
            metadata: HashMap::new(),
        }
    }

    /// Creates a new `Schema` from a sequence of `Field` values
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate arrow;
    /// # use arrow::datatypes::{Field, DataType, Schema};
    /// let field_a = Field::new("a", DataType::Int64, false);
    /// let field_b = Field::new("b", DataType::Boolean, false);
    ///
    /// let schema = Schema::new(vec![field_a, field_b]);
    /// ```
    pub fn new(fields: Vec<Field>) -> Self {
        Self::new_with_metadata(fields, HashMap::new())
    }
    /// Creates a new `Schema` from a sequence of `Field` values
    /// and adds additional metadata in form of key value pairs.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate arrow;
    /// # use arrow::datatypes::{Field, DataType, Schema};
    /// # use std::collections::HashMap;
    /// let field_a = Field::new("a", DataType::Int64, false);
    /// let field_b = Field::new("b", DataType::Boolean, false);
    ///
    /// let mut metadata: HashMap<String, String> = HashMap::new();
    /// metadata.insert("row_count".to_string(), "100".to_string());
    ///
    /// let schema = Schema::new_with_metadata(vec![field_a, field_b], metadata);
    /// ```
    pub fn new_with_metadata(
        fields: Vec<Field>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self { fields, metadata }
    }

    /// Returns an immutable reference of the vector of `Field` instances
    pub fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    /// Returns an immutable reference of a specific `Field` instance selected using an
    /// offset within the internal `fields` vector
    pub fn field(&self, i: usize) -> &Field {
        &self.fields[i]
    }

    /// Returns an immutable reference to the Map of custom metadata key-value pairs.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Look up a column by name and return a immutable reference to the column along with
    /// it's index
    pub fn column_with_name(&self, name: &str) -> Option<(usize, &Field)> {
        self.fields
            .iter()
            .enumerate()
            .find(|&(_, c)| c.name == name)
    }

    /// Generate a JSON representation of the `Schema`
    pub fn to_json(&self) -> Value {
        json!({
            "fields": self.fields.iter().map(|field| field.to_json()).collect::<Vec<Value>>(),
            "metadata": serde_json::to_value(&self.metadata).unwrap()
        })
    }

    /// Parse a `Schema` definition from a JSON representation
    pub fn from(json: &Value) -> Result<Self> {
        match *json {
            Value::Object(ref schema) => {
                let fields = if let Some(Value::Array(fields)) = schema.get("fields") {
                    fields
                        .iter()
                        .map(|f| Field::from(f))
                        .collect::<Result<_>>()?
                } else {
                    return Err(ArrowError::ParseError(
                        "Schema fields should be an array".to_string(),
                    ));
                };

                let metadata = if let Some(value) = schema.get("metadata") {
                    Self::from_metadata(value)?
                } else {
                    HashMap::default()
                };

                Ok(Self { fields, metadata })
            }
            _ => Err(ArrowError::ParseError(
                "Invalid json value type for schema".to_string(),
            )),
        }
    }

    /// Parse a `metadata` definition from a JSON representation
    fn from_metadata(json: &Value) -> Result<HashMap<String, String>> {
        if let Value::Object(md) = json {
            md.iter()
                .map(|(k, v)| {
                    if let Value::String(v) = v {
                        Ok((k.to_string(), v.to_string()))
                    } else {
                        Err(ArrowError::ParseError(
                            "metadata `value` field must be a string".to_string(),
                        ))
                    }
                })
                .collect::<Result<_>>()
        } else {
            Err(ArrowError::ParseError(
                "`metadata` field must be an object".to_string(),
            ))
        }
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            &self
                .fields
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

pub type SchemaRef = Arc<Schema>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use serde_json::Number;
    use serde_json::Value::{Bool, Number as VNumber};
    use std::f32::NAN;

    #[test]
    fn create_struct_type() {
        let _person = DataType::Struct(vec![
            Field::new("first_name", DataType::Utf8, false),
            Field::new("last_name", DataType::Utf8, false),
            Field::new(
                "address",
                DataType::Struct(vec![
                    Field::new("street", DataType::Utf8, false),
                    Field::new("zip", DataType::UInt16, false),
                ]),
                false,
            ),
        ]);
    }

    #[test]
    fn serde_struct_type() {
        let person = DataType::Struct(vec![
            Field::new("first_name", DataType::Utf8, false),
            Field::new("last_name", DataType::Utf8, false),
            Field::new(
                "address",
                DataType::Struct(vec![
                    Field::new("street", DataType::Utf8, false),
                    Field::new("zip", DataType::UInt16, false),
                ]),
                false,
            ),
        ]);

        let serialized = serde_json::to_string(&person).unwrap();

        // NOTE that this is testing the default (derived) serialization format, not the
        // JSON format specified in metadata.md

        assert_eq!(
            "{\"Struct\":[\
             {\"name\":\"first_name\",\"data_type\":\"Utf8\",\"nullable\":false,\"dict_id\":0,\"dict_is_ordered\":false},\
             {\"name\":\"last_name\",\"data_type\":\"Utf8\",\"nullable\":false,\"dict_id\":0,\"dict_is_ordered\":false},\
             {\"name\":\"address\",\"data_type\":{\"Struct\":\
             [{\"name\":\"street\",\"data_type\":\"Utf8\",\"nullable\":false,\"dict_id\":0,\"dict_is_ordered\":false},\
             {\"name\":\"zip\",\"data_type\":\"UInt16\",\"nullable\":false,\"dict_id\":0,\"dict_is_ordered\":false}\
             ]},\"nullable\":false,\"dict_id\":0,\"dict_is_ordered\":false}]}",
            serialized
        );

        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(person, deserialized);
    }

    #[test]
    fn struct_field_to_json() {
        let f = Field::new(
            "address",
            DataType::Struct(vec![
                Field::new("street", DataType::Utf8, false),
                Field::new("zip", DataType::UInt16, false),
            ]),
            false,
        );
        let value: Value = serde_json::from_str(
            r#"{
                "name": "address",
                "nullable": false,
                "type": {
                    "name": "struct"
                },
                "children": [
                    {
                        "name": "street",
                        "nullable": false,
                        "type": {
                            "name": "utf8"
                        },
                        "children": []
                    },
                    {
                        "name": "zip",
                        "nullable": false,
                        "type": {
                            "name": "int",
                            "bitWidth": 16,
                            "isSigned": false
                        },
                        "children": []
                    }
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(value, f.to_json());
    }

    #[test]
    fn primitive_field_to_json() {
        let f = Field::new("first_name", DataType::Utf8, false);
        let value: Value = serde_json::from_str(
            r#"{
                "name": "first_name",
                "nullable": false,
                "type": {
                    "name": "utf8"
                },
                "children": []
            }"#,
        )
        .unwrap();
        assert_eq!(value, f.to_json());
    }
    #[test]
    fn parse_struct_from_json() {
        let json = r#"
        {
            "name": "address",
            "type": {
                "name": "struct"
            },
            "nullable": false,
            "children": [
                {
                    "name": "street",
                    "type": {
                    "name": "utf8"
                    },
                    "nullable": false,
                    "children": []
                },
                {
                    "name": "zip",
                    "type": {
                    "name": "int",
                    "isSigned": false,
                    "bitWidth": 16
                    },
                    "nullable": false,
                    "children": []
                }
            ]
        }
        "#;
        let value: Value = serde_json::from_str(json).unwrap();
        let dt = Field::from(&value).unwrap();

        let expected = Field::new(
            "address",
            DataType::Struct(vec![
                Field::new("street", DataType::Utf8, false),
                Field::new("zip", DataType::UInt16, false),
            ]),
            false,
        );

        assert_eq!(expected, dt);
    }

    #[test]
    fn parse_utf8_from_json() {
        let json = "{\"name\":\"utf8\"}";
        let value: Value = serde_json::from_str(json).unwrap();
        let dt = DataType::from(&value).unwrap();
        assert_eq!(DataType::Utf8, dt);
    }

    #[test]
    fn parse_int32_from_json() {
        let json = "{\"name\": \"int\", \"isSigned\": true, \"bitWidth\": 32}";
        let value: Value = serde_json::from_str(json).unwrap();
        let dt = DataType::from(&value).unwrap();
        assert_eq!(DataType::Int32, dt);
    }

    #[test]
    fn schema_json() {
        // Add some custom metadata
        let metadata: HashMap<String, String> =
            [("Key".to_string(), "Value".to_string())]
                .iter()
                .cloned()
                .collect();

        let schema = Schema::new_with_metadata(
            vec![
                Field::new("c1", DataType::Utf8, false),
                Field::new("c2", DataType::Binary, false),
                Field::new("c3", DataType::FixedSizeBinary(3), false),
                Field::new("c4", DataType::Boolean, false),
                Field::new("c5", DataType::Date32(DateUnit::Day), false),
                Field::new("c6", DataType::Date64(DateUnit::Millisecond), false),
                Field::new("c7", DataType::Time32(TimeUnit::Second), false),
                Field::new("c8", DataType::Time32(TimeUnit::Millisecond), false),
                Field::new("c9", DataType::Time32(TimeUnit::Microsecond), false),
                Field::new("c10", DataType::Time32(TimeUnit::Nanosecond), false),
                Field::new("c11", DataType::Time64(TimeUnit::Second), false),
                Field::new("c12", DataType::Time64(TimeUnit::Millisecond), false),
                Field::new("c13", DataType::Time64(TimeUnit::Microsecond), false),
                Field::new("c14", DataType::Time64(TimeUnit::Nanosecond), false),
                Field::new("c15", DataType::Timestamp(TimeUnit::Second, None), false),
                Field::new(
                    "c16",
                    DataType::Timestamp(
                        TimeUnit::Millisecond,
                        Some(Arc::new("UTC".to_string())),
                    ),
                    false,
                ),
                Field::new(
                    "c17",
                    DataType::Timestamp(
                        TimeUnit::Microsecond,
                        Some(Arc::new("Africa/Johannesburg".to_string())),
                    ),
                    false,
                ),
                Field::new(
                    "c18",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new("c19", DataType::Interval(IntervalUnit::DayTime), false),
                Field::new("c20", DataType::Interval(IntervalUnit::YearMonth), false),
                Field::new("c21", DataType::List(Box::new(DataType::Boolean)), false),
                Field::new(
                    "c22",
                    DataType::FixedSizeList(Box::new(DataType::Boolean), 5),
                    false,
                ),
                Field::new(
                    "c23",
                    DataType::List(Box::new(DataType::List(Box::new(DataType::Struct(
                        vec![],
                    ))))),
                    true,
                ),
                Field::new(
                    "c24",
                    DataType::Struct(vec![
                        Field::new("a", DataType::Utf8, false),
                        Field::new("b", DataType::UInt16, false),
                    ]),
                    false,
                ),
                Field::new("c25", DataType::Interval(IntervalUnit::YearMonth), true),
                Field::new("c26", DataType::Interval(IntervalUnit::DayTime), true),
                Field::new("c27", DataType::Duration(TimeUnit::Second), false),
                Field::new("c28", DataType::Duration(TimeUnit::Millisecond), false),
                Field::new("c29", DataType::Duration(TimeUnit::Microsecond), false),
                Field::new("c30", DataType::Duration(TimeUnit::Nanosecond), false),
                Field::new_dict(
                    "c31",
                    DataType::Dictionary(
                        Box::new(DataType::Int32),
                        Box::new(DataType::Utf8),
                    ),
                    true,
                    123,
                    true,
                ),
            ],
            metadata,
        );

        let expected = schema.to_json();
        let json = r#"{
                "fields": [
                    {
                        "name": "c1",
                        "nullable": false,
                        "type": {
                            "name": "utf8"
                        },
                        "children": []
                    },
                    {
                        "name": "c2",
                        "nullable": false,
                        "type": {
                            "name": "binary"
                        },
                        "children": []
                    },
                    {
                        "name": "c3",
                        "nullable": false,
                        "type": {
                            "name": "fixedsizebinary",
                            "byteWidth": 3
                        },
                        "children": []
                    },
                    {
                        "name": "c4",
                        "nullable": false,
                        "type": {
                            "name": "bool"
                        },
                        "children": []
                    },
                    {
                        "name": "c5",
                        "nullable": false,
                        "type": {
                            "name": "date",
                            "unit": "DAY"
                        },
                        "children": []
                    },
                    {
                        "name": "c6",
                        "nullable": false,
                        "type": {
                            "name": "date",
                            "unit": "MILLISECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c7",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 32,
                            "unit": "SECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c8",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 32,
                            "unit": "MILLISECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c9",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 32,
                            "unit": "MICROSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c10",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 32,
                            "unit": "NANOSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c11",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 64,
                            "unit": "SECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c12",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 64,
                            "unit": "MILLISECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c13",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 64,
                            "unit": "MICROSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c14",
                        "nullable": false,
                        "type": {
                            "name": "time",
                            "bitWidth": 64,
                            "unit": "NANOSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c15",
                        "nullable": false,
                        "type": {
                            "name": "timestamp",
                            "unit": "SECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c16",
                        "nullable": false,
                        "type": {
                            "name": "timestamp",
                            "unit": "MILLISECOND",
                            "timezone": "UTC"
                        },
                        "children": []
                    },
                    {
                        "name": "c17",
                        "nullable": false,
                        "type": {
                            "name": "timestamp",
                            "unit": "MICROSECOND",
                            "timezone": "Africa/Johannesburg"
                        },
                        "children": []
                    },
                    {
                        "name": "c18",
                        "nullable": false,
                        "type": {
                            "name": "timestamp",
                            "unit": "NANOSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c19",
                        "nullable": false,
                        "type": {
                            "name": "interval",
                            "unit": "DAY_TIME"
                        },
                        "children": []
                    },
                    {
                        "name": "c20",
                        "nullable": false,
                        "type": {
                            "name": "interval",
                            "unit": "YEAR_MONTH"
                        },
                        "children": []
                    },
                    {
                        "name": "c21",
                        "nullable": false,
                        "type": {
                            "name": "list"
                        },
                        "children": [
                            {
                                "name": "item",
                                "nullable": false,
                                "type": {
                                    "name": "bool"
                                },
                                "children": []
                            }
                        ]
                    },
                    {
                        "name": "c22",
                        "nullable": false,
                        "type": {
                            "name": "fixedsizelist",
                            "listSize": 5
                        },
                        "children": [
                            {
                                "name": "item",
                                "nullable": false,
                                "type": {
                                    "name": "bool"
                                },
                                "children": []
                            }
                        ]
                    },
                    {
                        "name": "c23",
                        "nullable": true,
                        "type": {
                            "name": "list"
                        },
                        "children": [
                            {
                                "name": "item",
                                "nullable": true,
                                "type": {
                                    "name": "list"
                                },
                                "children": [
                                    {
                                        "name": "item",
                                        "nullable": true,
                                        "type": {
                                            "name": "struct"
                                        },
                                        "children": []
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "name": "c24",
                        "nullable": false,
                        "type": {
                            "name": "struct"
                        },
                        "children": [
                            {
                                "name": "a",
                                "nullable": false,
                                "type": {
                                    "name": "utf8"
                                },
                                "children": []
                            },
                            {
                                "name": "b",
                                "nullable": false,
                                "type": {
                                    "name": "int",
                                    "bitWidth": 16,
                                    "isSigned": false
                                },
                                "children": []
                            }
                        ]
                    },
                    {
                        "name": "c25",
                        "nullable": true,
                        "type": {
                            "name": "interval",
                            "unit": "YEAR_MONTH"
                        },
                        "children": []
                    },
                    {
                        "name": "c26",
                        "nullable": true,
                        "type": {
                            "name": "interval",
                            "unit": "DAY_TIME"
                        },
                        "children": []
                    },
                    {
                        "name": "c27",
                        "nullable": false,
                        "type": {
                            "name": "duration",
                            "unit": "SECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c28",
                        "nullable": false,
                        "type": {
                            "name": "duration",
                            "unit": "MILLISECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c29",
                        "nullable": false,
                        "type": {
                            "name": "duration",
                            "unit": "MICROSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c30",
                        "nullable": false,
                        "type": {
                            "name": "duration",
                            "unit": "NANOSECOND"
                        },
                        "children": []
                    },
                    {
                        "name": "c31",
                        "nullable": true,
                        "children": [],
                        "type": {
                          "name": "utf8"
                        },
                        "dictionary": {
                          "id": 123,
                          "indexType": {
                            "name": "int",
                            "isSigned": true,
                            "bitWidth": 32
                          },
                          "isOrdered": true
                        }
                    }
                ],
                "metadata" : {
                    "Key": "Value"
                }
            }"#;
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(expected, value);

        // convert back to a schema
        let value: Value = serde_json::from_str(&json).unwrap();
        let schema2 = Schema::from(&value).unwrap();

        assert_eq!(schema, schema2);

        // Check that empty metadata produces empty value in JSON and can be parsed
        let json = r#"{
                "fields": [
                    {
                        "name": "c1",
                        "nullable": false,
                        "type": {
                            "name": "utf8"
                        },
                        "children": []
                    }
                ],
                "metadata": {}
            }"#;
        let value: Value = serde_json::from_str(&json).unwrap();
        let schema = Schema::from(&value).unwrap();
        assert!(schema.metadata.is_empty());

        // Check that metadata field is not required in the JSON.
        let json = r#"{
                "fields": [
                    {
                        "name": "c1",
                        "nullable": false,
                        "type": {
                            "name": "utf8"
                        },
                        "children": []
                    }
                ]
            }"#;
        let value: Value = serde_json::from_str(&json).unwrap();
        let schema = Schema::from(&value).unwrap();
        assert!(schema.metadata.is_empty());
    }

    #[test]
    fn create_schema_string() {
        let _person = Schema::new(vec![
            Field::new("first_name", DataType::Utf8, false),
            Field::new("last_name", DataType::Utf8, false),
            Field::new(
                "address",
                DataType::Struct(vec![
                    Field::new("street", DataType::Utf8, false),
                    Field::new("zip", DataType::UInt16, false),
                ]),
                false,
            ),
        ]);
        assert_eq!(_person.to_string(), "first_name: Utf8, last_name: Utf8, address: Struct([Field { name: \"street\", data_type: Utf8, nullable: false, dict_id: 0, dict_is_ordered: false }, Field { name: \"zip\", data_type: UInt16, nullable: false, dict_id: 0, dict_is_ordered: false }])")
    }

    #[test]
    fn schema_field_accessors() {
        let _person = Schema::new(vec![
            Field::new("first_name", DataType::Utf8, false),
            Field::new("last_name", DataType::Utf8, false),
            Field::new(
                "address",
                DataType::Struct(vec![
                    Field::new("street", DataType::Utf8, false),
                    Field::new("zip", DataType::UInt16, false),
                ]),
                false,
            ),
        ]);

        // test schema accessors
        assert_eq!(_person.fields().len(), 3);

        // test field accessors
        assert_eq!(_person.fields()[0].name(), "first_name");
        assert_eq!(_person.fields()[0].data_type(), &DataType::Utf8);
        assert_eq!(_person.fields()[0].is_nullable(), false);
    }

    #[test]
    fn schema_equality() {
        let schema1 = Schema::new(vec![
            Field::new("c1", DataType::Utf8, false),
            Field::new("c2", DataType::Float64, true),
        ]);
        let schema2 = Schema::new(vec![
            Field::new("c1", DataType::Utf8, false),
            Field::new("c2", DataType::Float64, true),
        ]);

        assert_eq!(schema1, schema2);

        let schema3 = Schema::new(vec![
            Field::new("c1", DataType::Utf8, false),
            Field::new("c2", DataType::Float32, true),
        ]);
        let schema4 = Schema::new(vec![
            Field::new("C1", DataType::Utf8, false),
            Field::new("C2", DataType::Float64, true),
        ]);

        assert!(schema1 != schema3);
        assert!(schema1 != schema4);
        assert!(schema2 != schema3);
        assert!(schema2 != schema4);
        assert!(schema3 != schema4);
    }

    #[test]
    fn test_arrow_native_type_to_json() {
        assert_eq!(Some(Bool(true)), true.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1i8.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1i16.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1i32.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1i64.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1u8.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1u16.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1u32.into_json_value());
        assert_eq!(Some(VNumber(Number::from(1))), 1u64.into_json_value());
        assert_eq!(
            Some(VNumber(Number::from_f64(0.01 as f64).unwrap())),
            0.01.into_json_value()
        );
        assert_eq!(
            Some(VNumber(Number::from_f64(0.01f64).unwrap())),
            0.01f64.into_json_value()
        );
        assert_eq!(None, NAN.into_json_value());
    }
}
