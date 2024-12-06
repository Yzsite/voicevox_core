use std::{collections::HashMap, fmt::Display, marker::PhantomData, sync::Arc};

use enum_map::{Enum as _, EnumMap};
use itertools::Itertools as _;
use tracing::info;

use crate::error::ErrorRepr;

use super::{
    model_file, InferenceDomain, InferenceInputSignature, InferenceOperation, InferenceRuntime,
    InferenceSessionOptions, InferenceSignature, ParamInfo,
};

pub(crate) struct InferenceSessionSet<R: InferenceRuntime, D: InferenceDomain>(
    EnumMap<D::Operation, Arc<std::sync::Mutex<R::Session>>>,
);

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(crate) fn new(
        rt: &R,
        model_bytes: &EnumMap<D::Operation, Vec<u8>>,
        options: &EnumMap<D::Operation, InferenceSessionOptions>,
    ) -> anyhow::Result<Self> {
        let mut sessions = model_bytes
            .iter()
            .map(|(op, model_bytes)| {
                let (expected_input_param_infos, expected_output_param_infos) =
                    <D::Operation as InferenceOperation>::PARAM_INFOS[op];

                info!(
                    "Loading model for operation {:?} with options {:?}",
                    op, options[op]
                );
                let (sess, actual_input_param_infos, actual_output_param_infos) =
                    rt.new_session(|| model_file::decrypt(model_bytes), options[op])?;

                check_param_infos(expected_input_param_infos, &actual_input_param_infos)?;
                check_param_infos(expected_output_param_infos, &actual_output_param_infos)?;

                Ok((op.into_usize(), std::sync::Mutex::new(sess).into()))
            })
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        return Ok(Self(EnumMap::<D::Operation, _>::from_fn(|k| {
            sessions.remove(&k.into_usize()).expect("should exist")
        })));

        #[cfg(not(target_family = "wasm"))]
        fn check_param_infos<D: PartialEq + Display>(
            expected: &[ParamInfo<D>],
            actual: &[ParamInfo<D>],
        ) -> anyhow::Result<()> {
            use anyhow::bail;
            if !(expected.len() == actual.len()
                && itertools::zip_eq(expected, actual)
                    .all(|(expected, actual)| expected.accepts(actual)))
            {
                let expected = display_param_infos(expected);
                let actual = display_param_infos(actual);
                bail!("expected {{{expected}}}, got {{{actual}}}")
            }
            Ok(())
        }

        #[cfg(target_family = "wasm")]
        fn check_param_infos<D: PartialEq + Display>(
            _expected: &[ParamInfo<D>],
            _actual: &[ParamInfo<D>],
        ) -> anyhow::Result<()> {
            // onnxruntime webはパラメータ情報を返さないので、チェックを行わない
            Ok(())
        }

        fn display_param_infos(infos: &[ParamInfo<impl Display>]) -> impl Display {
            infos
                .iter()
                .map(|ParamInfo { name, dt, ndim }| {
                    let brackets = match *ndim {
                        Some(ndim) => &"[]".repeat(ndim),
                        None => "[]...",
                    };
                    format!("{name}: {dt}{brackets}")
                })
                .join(", ")
        }
    }
}

impl<R: InferenceRuntime, D: InferenceDomain> InferenceSessionSet<R, D> {
    pub(crate) fn get<I>(&self) -> InferenceSessionCell<R, I>
    where
        I: InferenceInputSignature<Signature: InferenceSignature<Domain = D>>,
    {
        InferenceSessionCell {
            inner: self.0[I::Signature::OPERATION].clone(),
            marker: PhantomData,
        }
    }
}

pub(crate) struct InferenceSessionCell<R: InferenceRuntime, I> {
    inner: Arc<std::sync::Mutex<R::Session>>,
    marker: PhantomData<fn(I)>,
}

impl<R: InferenceRuntime, I: InferenceInputSignature> InferenceSessionCell<R, I> {
    pub(crate) fn run(
        self,
        input: I,
    ) -> crate::Result<<I::Signature as InferenceSignature>::Output> {
        let inner = &mut self.inner.lock().unwrap();
        (|| R::run(input.make_run_context::<R>(inner)?)?.try_into())()
            .map_err(ErrorRepr::RunModel)
            .map_err(Into::into)
    }
}