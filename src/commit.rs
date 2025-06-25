use crate::setup::PublicKey;
use anyhow::anyhow;
use ark_bls12_381::Fr;
use ark_ec::CurveGroup;
use ark_poly::{Evaluations, GeneralEvaluationDomain};

pub fn commit<T>(pk: &PublicKey<T::Affine>, a: &[T::ScalarField]) -> anyhow::Result<T>
where
    T: CurveGroup<ScalarField = Fr>,
{
    T::msm(&pk.pk[..a.len()], a).map_err(|e| anyhow!("Failed to commit with error: {e}."))
}

pub fn interpolate_and_commit<T>(
    domain: GeneralEvaluationDomain<T::ScalarField>,
    pk: &PublicKey<T::Affine>,
    a: &[T::ScalarField],
) -> anyhow::Result<T>
where
    T: CurveGroup<ScalarField = Fr>,
{
    let vec = Evaluations::from_vec_and_domain(a.to_vec(), domain).interpolate();
    commit(pk, &vec)
}
