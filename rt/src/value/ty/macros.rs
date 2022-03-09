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

#[macro_export]// static mut NULL_PARENT: MaybeUninit<Base<Scope>> = MaybeUninit::uninit();
macro_rules! _handle_quest_type_attrs {
	($builder:expr, $name:literal, $body:expr) => {
		$builder.set_attr($name, $crate::Value::from(RustFn_new!($name, $body)).any())
			.expect(concat!("error initializing ", stringify!($ty), " for attr: ", stringify!($name)));
	};
}

#[macro_export]
macro_rules! quest_type_attrs {
	(for $type:ty $(where {$($gens:tt)*})?; 
		$($name:literal => $body:expr),+
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
					is_first_init = true;
					$crate::value::ty::scope::Builder::with_capacity(_length_of!($($name)*))
						.build(Default::default())
				});

				#[allow(unused_mut)]
				if is_first_init {
					let mut parent_mut = parent.as_mut().unwrap();
					$(
						_handle_quest_type_attrs!(parent_mut.header_mut(), $name, $body);
					)*
				}

				crate::Value::from(parent).any()
			}
		}
	};
}
