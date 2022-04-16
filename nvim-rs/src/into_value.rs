/// IntoValue provides proxu to `rmpv::Value::from`. This is required because
/// `rmpv` doesn't provide from method for `Vec<T: Into<Value>>`.
pub trait IntoValue {
    fn into_value(self) -> rmpv::Value;
}

macro_rules! impl_into_value {
    ($t:ty) => {
        impl IntoValue for $t {
            fn into_value(self) -> rmpv::Value {
                rmpv::Value::from(self)
            }
        }
    };
}

impl_into_value!(f64);
impl_into_value!(i64);
impl_into_value!(String);
impl_into_value!(bool);
impl_into_value!(rmpv::Value);

impl<T> IntoValue for Vec<T>
where
    T: Into<rmpv::Value>,
{
    fn into_value(self) -> rmpv::Value {
        self.into_iter().collect()
    }
}

impl<T> IntoValue for (T, T)
where
    T: Into<rmpv::Value>,
{
    fn into_value(self) -> rmpv::Value {
        rmpv::Value::from(vec![self.0.into(), self.1.into()])
    }
}
