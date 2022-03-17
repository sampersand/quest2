// #[macro_export]
// macro_rules! new_object {

// 	(
// 		with
// 			$(@data $data:expr;)?
// 			$(@parent $parent:ty;)?
// 			$(@parents [$($parents:ty)*];)?
// 		$($attr:expr => $kind:ident $value:expr),* $(,)?
// 	) => {

// 	};

// 	($($attr:expr => $kind:ident $value:expr),* $(,)?) => {
// 		new_object!(; $($attr => $kind $value)*)
// 	};
// }

#[macro_export]
macro_rules! create_class {
	($name:expr $(, parent $parent:expr)?; $($attr:expr => $kind:ident $value:expr),* $(,)?) => {(|| -> $crate::Result<$crate::AnyValue> {
		#[allow(unused_imports)]
		use $crate::value::{AsAny, Intern};

		#[allow(unused_mut)]
		let mut builder = $crate::value::ty::Class::builder($name, _length_of!($($attr)*));

		$({
			use $crate::value::ty::*;
			builder.parent($parent);
		})?

		$(
			builder.set_attr($attr, $crate::RustFn_new!($attr, $kind $value).as_any())?;
		)*

		Ok(builder.finish().as_any())
	})().expect(concat!("Class creation for '", $name, "' failed!"))}
}

#[macro_export]
macro_rules! new_quest_scope {
	($($attr:expr => $value:expr),* $(,)?) => {
		new_quest_scope!(@parent Object; $($attr => $value),*)
	};

	($(@parentof $child:ty;)?
	 $(@parent $parent:ty;)?
	 $(@parents [$($parents:ty)*];)?
	 $($attr:expr => $value:expr),*
	 $(,)?
	) => { (|| -> $crate::Result<_> {
			#[allow(unused_mut)]
			let mut builder = {
				#[allow(unused_imports)]
				use $crate::value::ty::*;
				$crate::value::ty::scope::Builder::with_capacity(_length_of!($($attr)*))
					$(.parent(<$parent>::instance()))?
					$(.parents($crate::value::ty::List::from_slice(&[
						$(<$parents>::instance()),*
					])))?
			};

			{
				#[allow(unused_macros)]
				macro_rules! method {
					($fn:ident) => (method!($($child)?, $fn));
					($type:ty, $fn:ident) => (func!(|this: AnyValue, args| {
						let this = this.downcast::<$type>()
							.ok_or_else(|| $crate::Error::InvalidTypeGiven {
								expected: <$type as $crate::value::NamedType>::TYPENAME,
								given: this.typename()
							})?;
						this.$fn(args)
					}));
				}

				#[allow(unused_macros)]
				macro_rules! func {
					($fn:expr) => (|args| {
						let (this, args) = args.split_first()?;
						$fn(this, args)
					});
				}

				#[allow(unused_imports)]
				use $crate::value::Intern;

				$(
					builder.set_attr($attr, RustFn_new!($attr, justargs $value).as_any())?;
				)*
			}

			Ok(builder.build(Default::default()))
	})()};
}

#[macro_export]
macro_rules! singleton_object {
	(for $type:ty
			$(where {$($gens:tt)*})?
			$(, parentof $child:ty)?
			$(, parent $parent:ty)?
			$(, parents [$($parents:ty),* $(,)?])?
			$(, late_binding_parent $late_binding_parent:ty)?
		;
		$($attr:expr => $value:expr),*
		$(,)?
	) => {
		impl<$($gens),*> $type {
			pub fn instance() -> $crate::AnyValue {
				use ::once_cell::sync::OnceCell;
				use $crate::value::AsAny;

				static INSTANCE: OnceCell<$crate::AnyValue> = OnceCell::new();

				let mut is_first_init = false;

				let ret = *INSTANCE.get_or_init(|| { is_first_init = true; new_quest_scope!{
					$(@parentof $child;)?
					$(@parent $parent;)?
					$(@parents [$($parents),*];)?
					$($attr => $value),*
				}.unwrap().as_any()});

				if is_first_init {
					$(
						#[allow(unused_imports)]
						use $crate::value::ty::*;
						ret.downcast::<$crate::value::Gc<$type>>()
							.unwrap()
							.as_mut()
							.unwrap()
							.header_mut()
							.set_singular_parent(<$late_binding_parent>::instance());
					)?
				}

				ret
			}
		}
		$(
			impl $crate::value::base::HasDefaultParent for $child {
				fn parent() -> $crate::AnyValue {
					<$type>::instance()
				}
			}
		)?

	}
}

#[macro_export]
macro_rules! quest_type {
	(
		$(#[$meta:meta])*
		$vis:vis struct $name:ident $(<$($gen:ident),*>)? ($($inner:tt)*) $(where {$($cond:tt)*})?;
	) => {
		$(#[$meta])*
		#[repr(transparent)]
		$vis struct $name $(<$($gen)*>)?($crate::value::base::Base<$($inner)*>) $(where $($cond)*)?;

		unsafe impl $(<$($gen),*>)? $crate::value::gc::Allocated for $name $(<$($gen),*>)?
		$(where $($cond)*)? {
			#[inline(always)]
			fn _inner_typeid() -> ::std::any::TypeId {
				::std::any::TypeId::of::<$($inner)*>()
			}
		}
	};
}

#[macro_export]
macro_rules! quest_type_alias {
	(
		$(#[$meta:meta])*
		$vis:vis struct $name:ident $(<$($gen:ident),*>)? ($($inner:tt)*) $(where {$($cond:tt)*})?;
	) => {
		$(#[$meta])*
		#[repr(transparent)]
		$vis struct $name $(<$($gen)*>)?($($inner)*) $(where $($cond)*)?;

		unsafe impl $(<$($gen),*>)? $crate::value::gc::Allocated for $name $(<$($gen),*>)?
		$(where $($cond)*)? {
			#[inline(always)]
			fn _inner_typeid() -> ::std::any::TypeId {
				::std::any::TypeId::of::<$name $(<$($gen),*>)?>()
			}
		}
	};
}

#[macro_export]
macro_rules! _length_of {
	() => (0);
	($_tt:tt $($rest:tt)*) => (1+_length_of!($($rest)*));
}

#[macro_export]
macro_rules! _handle_quest_type_attrs {
	($ty:ty, $builder:expr, $name:expr, meth $func:expr) => {
		_handle_quest_type_attrs!($ty, $builder, $name, func |args| {
			let (this, args) = args.split_first()?;
			let this = this.downcast::<$ty>().ok_or_else(|| $crate::Error::InvalidTypeGiven {
				expected: <$ty as $crate::value::NamedType>::TYPENAME,
				given: this.typename()
			})?;
			$func(this, args)
		})

		// $builder.set_attr($name, $crate::Value::from(RustFn_new!($name, |obj, args| {
		// })).any())
		// 	.expect(concat!("error initializing ", stringify!($ty), " for attr: ", stringify!($name)));
	};
	($ty:ty, $builder:expr, $name:expr, func $func:expr) => {
		$builder.set_attr($name, $crate::Value::from(RustFn_new!($name, justargs $func)).any())
			.expect(concat!("error initializing ", stringify!($ty), " for attr: ", stringify!($name)));
	};
}

#[macro_export]
macro_rules! quest_type_attrs {
	(
		for $type:ty
			$(where {$($gens:tt)*})?
			$(, parent $parent:ty)?
			$(, parents [$($parents:ty),* $(,)?])?
			$(, late_binding_parent $late_binding_parent:ty)?; 
		$($name:ident => $func_kind:ident $func:expr),*
		$(,)?
	) => {
		impl $crate::value::base::HasDefaultParent for $type {
			fn parent() -> $crate::AnyValue {
				#[allow(unused_imports)]
				use $crate::value::{AsAny, gc::Allocated};
				static PARENT: ::once_cell::sync::OnceCell<$crate::value::gc::Gc<$crate::value::ty::Scope>>
					= ::once_cell::sync::OnceCell::new();

				let mut is_first_init = false;
				let parent = *PARENT.get_or_init(|| {
					#[allow(unused_imports)]
					use $crate::value::ty::*;
					is_first_init = true;

					$crate::value::ty::scope::Builder::with_capacity(_length_of!($($name)*))
						$(.parent(<$parent>::instance()))?
						$(.parents($crate::value::ty::List::from_slice(&[
							$(<$parents>::instance()),*
						])))?
						.build(Default::default())
				});

				if !is_first_init {
					// dbg!(concat!("!is_first_init: ", stringify!($type)));
				}

				if is_first_init {
					// dbg!(concat!("is_first_init: ", stringify!($type)));
					#[allow(unused_macros)]
					macro_rules! method {
						($fn:expr) => (func!(|this: AnyValue, args| {
							let this = this.downcast::<$type>()
								.ok_or_else(|| $crate::Error::InvalidTypeGiven {
									expected: <$type as $crate::value::NamedType>::TYPENAME,
									given: this.typename()
								})?;
							$fn(this, args)
						}));
					}

					#[allow(unused_macros)]
					macro_rules! func {
						($fn:expr) => (|args| {
							let (this, args) = args.split_first()?;
							$fn(this, args)
						});
					}

					#[allow(unused_mut,unused_variables)]
					let mut parent_mut = parent.as_mut().unwrap();
					$(
						_handle_quest_type_attrs!($type, parent_mut.header_mut(), $crate::value::Intern::$name, $func_kind $func);
					)*
					$(
						unsafe {
							#[allow(unused_imports)]
							use $crate::value::ty::*;
							parent_mut._set_parent_to(<$late_binding_parent>::instance())
						}
					)?
				}

				crate::Value::from(parent).any()
			}
		}
	};
}

