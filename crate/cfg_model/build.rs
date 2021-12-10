fn main() {
    use std::{env, fs, io::Write, path::Path};

    use common::{generate_impls_for_n_args, ArgExprs};

    let out_dir = env::var_os("OUT_DIR").expect("Failed to read `OUT_DIR` environment variable.");
    let station_fn_dir = Path::new(&out_dir).join("station_fn");
    fs::create_dir_all(&station_fn_dir).expect("Failed to create `station_fn_dir`.");

    let mut station_fn_metadata_ext =
        common::open_impl_file(&station_fn_dir, "station_fn_metadata_ext.rs");
    let mut station_fn_resource_impl =
        common::open_impl_file(&station_fn_dir, "station_fn_resource_impl.rs");
    let mut station_fn_res_impl = common::open_impl_file(&station_fn_dir, "station_fn_res_impl.rs");

    let mut write_fn = |arg_exprs: ArgExprs<'_>| {
        station_fn_metadata_ext::write_station_fn_metadata_ext(
            &mut station_fn_metadata_ext,
            arg_exprs,
        );

        station_fn_resource_impl::write_station_fn_resource_impl(
            &mut station_fn_resource_impl,
            arg_exprs,
        );

        station_fn_res_impl::write_station_fn_res_impl(&mut station_fn_res_impl, arg_exprs);
    };

    generate_impls_for_n_args::<_, 1>(&mut write_fn);
    generate_impls_for_n_args::<_, 2>(&mut write_fn);
    generate_impls_for_n_args::<_, 3>(&mut write_fn);
    generate_impls_for_n_args::<_, 4>(&mut write_fn);
    generate_impls_for_n_args::<_, 5>(&mut write_fn);
    generate_impls_for_n_args::<_, 6>(&mut write_fn);

    station_fn_resource_impl
        .flush()
        .expect("Failed to flush writer for station_fn_resource_impl.rs");

    println!("cargo:rerun-if-changed=build.rs");
}

mod common {
    use std::{
        fmt::Write as _,
        fs::{File, OpenOptions},
        io::BufWriter,
        mem::MaybeUninit,
        path::Path,
    };

    #[derive(Clone, Copy, Debug)]
    pub struct ArgExprs<'s> {
        pub args_csv: &'s str,
        pub arg_refs_csv: &'s str,
        pub arg_bounds_list: &'s str,
        pub resource_arg_borrows: &'s str,
        pub resource_arg_try_borrows: &'s str,
        pub resource_arg_vars: &'s str,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Ref {
        Immutable,
        Mutable,
    }

    pub fn open_impl_file(out_dir: &Path, file_name: &str) -> BufWriter<File> {
        let station_fn_resource_impl_path = out_dir.join(file_name);
        let station_fn_resource_impl = OpenOptions::new()
            .create(true)
            .write(true)
            .open(station_fn_resource_impl_path)
            .unwrap_or_else(|e| panic!("Failed to open `{}`. Error: {}", file_name, e));
        BufWriter::new(station_fn_resource_impl)
    }

    pub fn generate_impls_for_n_args<FnWrite, const N: usize>(fn_write: &mut FnWrite)
    where
        FnWrite: FnMut(ArgExprs<'_>),
    {
        // "A0, A1"
        let args_csv = args_csv::<N>();

        // "    A0: 'static,\n    A1: 'static,"
        let arg_bounds_list = arg_bounds_list::<N>();

        arg_refs_combinations::<N>().for_each(|arg_refs| {
            let mut arg_refs_iter = arg_refs.iter().copied().enumerate();

            // &mut A0, &A1
            let arg_refs_csv = {
                let mut arg_refs_csv = String::with_capacity(N * 8);
                if let Some((_index, arg_ref_first)) = arg_refs_iter.next() {
                    match arg_ref_first {
                        Ref::Immutable => arg_refs_csv.push_str("&A0"),
                        Ref::Mutable => arg_refs_csv.push_str("&mut A0"),
                    }
                }

                if N == 1 {
                    arg_refs_csv.push(',');
                } else {
                    arg_refs_iter
                        .try_for_each(|(index, arg_ref)| match arg_ref {
                            Ref::Immutable => write!(&mut arg_refs_csv, ", &A{}", index),
                            Ref::Mutable => write!(&mut arg_refs_csv, ", &mut A{}", index),
                        })
                        .expect("Failed to append to `arg_refs_csv` string.");
                }

                arg_refs_csv
            };

            // let a0 = train_report.borrow::<A0>();
            // let mut a1 = train_report.borrow_mut::<A1>();
            // ..
            let resource_arg_borrows = resource_arg_borrows(arg_refs);
            let resource_arg_try_borrows = resource_arg_try_borrows(arg_refs);

            // &*a0, &mut *a1
            let resource_arg_vars = resource_arg_vars::<N>(arg_refs);

            let args_csv = args_csv.as_str();
            let arg_refs_csv = arg_refs_csv.as_str();
            let arg_bounds_list = arg_bounds_list.as_str();
            let resource_arg_borrows = resource_arg_borrows.as_str();
            let resource_arg_try_borrows = resource_arg_try_borrows.as_str();
            let resource_arg_vars = resource_arg_vars.as_str();

            let arg_exprs = ArgExprs {
                args_csv,
                arg_refs_csv,
                arg_bounds_list,
                resource_arg_borrows,
                resource_arg_try_borrows,
                resource_arg_vars,
            };

            fn_write(arg_exprs);
        })
    }

    fn resource_arg_vars<const N: usize>(arg_refs: [Ref; N]) -> String {
        let mut resource_arg_vars = String::with_capacity(N * 10);
        let mut arg_refs_iter = arg_refs.iter().copied().enumerate();
        if let Some((index, arg_ref)) = arg_refs_iter.next() {
            match arg_ref {
                Ref::Immutable => write!(&mut resource_arg_vars, "&*a{}", index),
                Ref::Mutable => write!(&mut resource_arg_vars, "&mut *a{}", index),
            }
            .expect("Failed to append to `resource_arg_vars` string.")
        }
        arg_refs_iter
            .try_for_each(|(index, arg_ref)| match arg_ref {
                Ref::Immutable => write!(&mut resource_arg_vars, ", &*a{}", index),
                Ref::Mutable => write!(&mut resource_arg_vars, ", &mut *a{}", index),
            })
            .expect("Failed to append to `resource_arg_vars` string.");
        resource_arg_vars
    }

    fn resource_arg_borrows<const N: usize>(arg_refs: [Ref; N]) -> String {
        let mut resource_arg_borrows = String::with_capacity(N * 44);
        let mut arg_refs_iter = arg_refs.iter().copied().enumerate();
        arg_refs_iter
            .try_for_each(|(index, arg_ref)| match arg_ref {
                Ref::Immutable => writeln!(
                    &mut resource_arg_borrows,
                    "let a{index} = train_report.borrow::<A{index}>();",
                    index = index
                ),
                Ref::Mutable => writeln!(
                    &mut resource_arg_borrows,
                    "let mut a{index} = train_report.borrow_mut::<A{index}>();",
                    index = index
                ),
            })
            .expect("Failed to append to `resource_arg_borrows` string.");
        resource_arg_borrows
    }

    fn resource_arg_try_borrows<const N: usize>(arg_refs: [Ref; N]) -> String {
        let mut resource_arg_try_borrows = String::with_capacity(N * 44);
        let mut arg_refs_iter = arg_refs.iter().copied().enumerate();
        arg_refs_iter
            .try_for_each(|(index, arg_ref)| match arg_ref {
                Ref::Immutable => writeln!(
                    &mut resource_arg_try_borrows,
                    "let a{index} = train_report.try_borrow::<A{index}>()?;",
                    index = index
                ),
                Ref::Mutable => writeln!(
                    &mut resource_arg_try_borrows,
                    "let mut a{index} = train_report.try_borrow_mut::<A{index}>()?;",
                    index = index
                ),
            })
            .expect("Failed to append to `resource_arg_try_borrows` string.");
        resource_arg_try_borrows
    }

    fn arg_refs_combinations<const N: usize>() -> impl Iterator<Item = [Ref; N]> {
        (0..(2 << (N - 1))).map(|m| {
            // `m` is the combination variation count.
            // Whether an argument is immutable or mutable is bed on its corresponding bit
            // value of `m`.

            // Create an uninitialized array of `MaybeUninit`. The `assume_init` is safe
            // because the type we are claiming to have initialized here is a bunch of
            // `MaybeUninit`s, which do not require initialization.
            //
            // https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
            //
            // Switch this to `MaybeUninit::uninit_array` once it is stable.
            let mut arg_refs: [MaybeUninit<Ref>; N] =
                unsafe { MaybeUninit::uninit().assume_init() };

            arg_refs
                .iter_mut()
                .enumerate()
                .for_each(move |(arg_n, arg_ref_mem)| {
                    // for N = 5
                    // m can be 0..32
                    // if 31 >> 5 is 0

                    if m >> arg_n & 1 == 0 {
                        arg_ref_mem.write(Ref::Immutable);
                    } else {
                        arg_ref_mem.write(Ref::Mutable);
                    }
                });

            // Everything is initialized. Transmute the array to the initialized type.
            // Unfortunately we cannot use this, see the following issues:
            //
            // * <https://github.com/rust-lang/rust/issues/61956>
            // * <https://github.com/rust-lang/rust/issues/80908>
            //
            // let arg_refs = unsafe { mem::transmute::<_, [Ref;
            // N]>(arg_refs) };

            #[allow(clippy::let_and_return)] // for clarity with `unsafe`
            let arg_refs = {
                let ptr = &mut arg_refs as *mut _ as *mut [Ref; N];
                let array = unsafe { ptr.read() };

                // We don't have to `mem::forget` the original because `Ref` is `Copy`.
                // mem::forget(arg_refs);

                array
            };

            arg_refs
        })
    }

    fn arg_bounds_list<const N: usize>() -> String {
        let mut arg_bounds_list = String::with_capacity(N * 50);
        #[cfg(feature = "debug")]
        arg_bounds_list.push_str("    A0: std::fmt::Debug + Send + Sync + 'static,");

        #[cfg(not(feature = "debug"))]
        arg_bounds_list.push_str("    A0: Send + Sync + 'static,");
        (1..N).fold(arg_bounds_list, |mut arg_bounds_list, n| {
            #[cfg(feature = "debug")]
            write!(
                &mut arg_bounds_list,
                "\n    A{}: std::fmt::Debug + Send + Sync + 'static,",
                n
            )
            .expect("Failed to append to args_csv string.");

            #[cfg(not(feature = "debug"))]
            write!(&mut arg_bounds_list, "\n    A{}: Send + Sync + 'static,", n)
                .expect("Failed to append to args_csv string.");
            arg_bounds_list
        })
    }

    fn args_csv<const N: usize>() -> String {
        let mut args_csv = String::with_capacity(N * 4);
        args_csv.push_str("A0");
        (1..N).fold(args_csv, |mut args_csv, n| {
            write!(&mut args_csv, ", A{}", n).expect("Failed to append to args_csv string.");
            args_csv
        })
    }
}

mod station_fn_metadata_ext {
    use std::{
        fs::File,
        io::{BufWriter, Write},
    };

    use super::common::ArgExprs;

    pub fn write_station_fn_metadata_ext(
        station_fn_metadata_ext: &mut BufWriter<File>,
        arg_exprs: ArgExprs<'_>,
    ) {
        let ArgExprs {
            args_csv,
            arg_refs_csv,
            arg_bounds_list,
            ..
        } = arg_exprs;

        write!(
            station_fn_metadata_ext,
            r#"
impl<Fun, R, E, {args_csv}> StationFnMetadataExt<Fun, R, E, ({arg_refs_csv})> for Fun
where
    Fun: for<'f> FnOnce(&'f mut StationMut<'_, E>, {arg_refs_csv}) -> StationFnReturn<'f, R, E> + 'static,
    {arg_bounds_list}
{{
    fn metadata<'f>(&self) -> FnMetadata<Fun, StationFnReturn<'f, R, E>, ({arg_refs_csv})> {{
        FnMetadata(PhantomData)
    }}
}}
"#,
            args_csv = args_csv,
            arg_refs_csv = arg_refs_csv,
            arg_bounds_list = arg_bounds_list,
        )
        .expect("Failed to append to station_fn_metadata_ext.");
    }
}

mod station_fn_resource_impl {
    use std::{
        fs::File,
        io::{BufWriter, Write},
    };

    use super::common::ArgExprs;

    pub fn write_station_fn_resource_impl(
        station_fn_resource_impl: &mut BufWriter<File>,
        arg_exprs: ArgExprs<'_>,
    ) {
        let ArgExprs {
            args_csv,
            arg_refs_csv,
            arg_bounds_list,
            resource_arg_borrows,
            resource_arg_try_borrows,
            resource_arg_vars,
        } = arg_exprs;

        write!(
            station_fn_resource_impl,
            r#"
impl<Fun, R, E, {args_csv}> StationFnResource<Fun, R, E, ({arg_refs_csv})>
where
    Fun: for<'f> Fn(&'f mut StationMut<'_, E>, {arg_refs_csv}) -> StationFnReturn<'f, R, E> + 'static,
    {arg_bounds_list}
{{
    pub fn call<'f>(&self, station: &'f mut StationMut<'_, E>, train_report: &TrainReport<E>) -> StationFnReturn<'f, R, E> {{
        {resource_arg_borrows}

        (self.func)(station, {resource_arg_vars})
    }}

    pub fn try_call<'f>(&self, station: &'f mut StationMut<'_, E>, train_report: &TrainReport<E>) -> Result<StationFnReturn<'f, R, E>, BorrowFail> {{
        {resource_arg_try_borrows}

        let ret_value = (self.func)(station, {resource_arg_vars});
        Ok(ret_value)
    }}
}}
"#,
            args_csv = args_csv,
            arg_refs_csv = arg_refs_csv,
            arg_bounds_list = arg_bounds_list,
            resource_arg_borrows = resource_arg_borrows,
            resource_arg_try_borrows = resource_arg_try_borrows,
            resource_arg_vars = resource_arg_vars,
        )
        .expect("Failed to write to station_fn_resource_impl.rs");
    }
}

mod station_fn_res_impl {
    use std::{
        fs::File,
        io::{BufWriter, Write},
    };

    use super::common::ArgExprs;

    pub fn write_station_fn_res_impl(
        fn_resource_impl: &mut BufWriter<File>,
        arg_exprs: ArgExprs<'_>,
    ) {
        let ArgExprs {
            args_csv,
            arg_refs_csv,
            arg_bounds_list,
            ..
        } = arg_exprs;

        write!(
            fn_resource_impl,
            r#"
impl<Fun, R, E, {args_csv}> StationFnRes<R, E> for StationFnResource<Fun, R, E, ({arg_refs_csv})>
where
    Fun: for<'f> Fn(&'f mut StationMut<'_, E>, {arg_refs_csv}) -> StationFnReturn<'f, R, E> + 'static,
    {arg_bounds_list}
{{
    fn call<'f>(&self, station: &'f mut StationMut<'_, E>, train_report: &TrainReport<E>) -> StationFnReturn<'f, R, E> {{
        Self::call(self, station, train_report)
    }}

    fn try_call<'f>(&self, station: &'f mut StationMut<'_, E>, train_report: &TrainReport<E>) -> Result<StationFnReturn<'f, R, E>, BorrowFail> {{
        Self::try_call(self, station, train_report)
    }}
}}
"#,
            args_csv = args_csv,
            arg_refs_csv = arg_refs_csv,
            arg_bounds_list = arg_bounds_list,
        )
        .expect("Failed to write to station_fn_res_impl.rs");
    }
}
