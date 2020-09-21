use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use std::fmt;

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum BevelFilterType {
    Inner,
    Outer,
    Full,
}

impl From<&str> for BevelFilterType {
    fn from(value: &str) -> Self {
        match value {
            "inner" => BevelFilterType::Inner,
            "outer" => BevelFilterType::Outer,
            "full" => BevelFilterType::Full,
            _ => BevelFilterType::Full,
        }
    }
}

impl From<BevelFilterType> for &str {
    fn from(v: BevelFilterType) -> Self {
        match v {
            BevelFilterType::Inner => "inner",
            BevelFilterType::Outer => "outer",
            BevelFilterType::Full => "full",
        }
    }
}

/// A BevelFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct BevelFilterObject<'gc>(GcCell<'gc, BevelFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct BevelFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    //TODO: is this an int
    angle: f64,
    blur_x: f64,
    blur_y: f64,
    distance: f64,
    highlight_alpha: f64,
    highlight_color: i32,
    knockout: bool,
    quality: i32,
    shadow_alpha: f64,
    shadow_color: i32,
    strength: f64,
    type_: BevelFilterType,
}

impl fmt::Debug for BevelFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("BevelFilter")
            .field("angle", &this.angle)
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("distance", &this.distance)
            .field("highlightAlpha", &this.highlight_alpha)
            .field("highlightColor", &this.highlight_color)
            .field("knockout", &this.knockout)
            .field("quality", &this.quality)
            .field("shadowAlpha", &this.shadow_alpha)
            .field("strength", &this.strength)
            .field("type", &this.type_)
            .finish()
    }
}

impl<'gc> BevelFilterObject<'gc> {
    add_field_accessors!(
        [set_angle, get_angle, angle, f64],
        [set_blur_x, get_blur_x, blur_x, f64],
        [set_blur_y, get_blur_y, blur_y, f64],
        [set_distance, get_distance, distance, f64],
        [
            set_highlight_alpha,
            get_highlight_alpha,
            highlight_alpha,
            f64
        ],
        [
            set_highlight_color,
            get_highlight_color,
            highlight_color,
            i32
        ],
        [set_knockout, get_knockout, knockout, bool],
        [set_quality, get_quality, quality, i32],
        [set_shadow_alpha, get_shadow_alpha, shadow_alpha, f64],
        [set_shadow_color, get_shadow_color, shadow_color, i32],
        [set_strength, get_strength, strength, f64],
        [set_type, get_type, type_, BevelFilterType], //TODO: type
    );

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        BevelFilterObject(GcCell::allocate(
            gc_context,
            BevelFilterData {
                base: ScriptObject::object(gc_context, proto),
                angle: 45.0,
                blur_x: 4.0,
                blur_y: 4.0,
                distance: 4.0, // TODO: check if int
                highlight_alpha: 1.0,
                highlight_color: 0xFFFFFF.into(), //TODO: int?
                knockout: false,
                quality: 1, //TODO: must be int!!
                shadow_alpha: 1.0,
                shadow_color: 0x000000.into(), //TODO: is int?
                strength: 1.0,                 // Int?
                type_: BevelFilterType::Inner,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for BevelFilterObject<'gc> {
    impl_custom_object_without_set!(base);

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        let base = self.0.read().base;
        base.internal_set(
            name,
            value,
            activation,
            (*self).into(),
            Some(activation.context.avm1.prototypes.bevel_filter),
        )
    }

    fn as_bevel_filter_object(&self) -> Option<BevelFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(BevelFilterObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.bevel_filter),
        )
        .into())
    }
}
