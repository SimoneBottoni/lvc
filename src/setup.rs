use crate::commit::commit;
use crate::util::{ark_de, ark_se};
use anyhow::anyhow;
use ark_bls12_381::{Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{AffineRepr, PrimeGroup, ScalarMul};
use ark_ff::Field;
use ark_poly::univariate::DensePolynomial;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_std::UniformRand;
use rand::thread_rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Setup {
    pub pk_g1: PublicKey<G1Affine>,
    pub pk_g2: PublicKey<G2Affine>,
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    pub vanishing_tau: G2Projective,
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    pub domain: GeneralEvaluationDomain<Fr>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublicKey<T>
where
    T: AffineRepr,
{
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    pub pk: Vec<T>,
}

impl<T> PublicKey<T>
where
    T: AffineRepr,
{
    pub const fn new(pk: Vec<T>) -> Self {
        Self { pk }
    }
}

impl Setup {
    pub fn build(n: usize) -> anyhow::Result<Self> {
        let n = n.next_power_of_two();
        let tau = Fr::rand(&mut thread_rng());

        let (pk_g1, pk_g2) = Self::build_public_keys(n, &tau);
        let domain = GeneralEvaluationDomain::<Fr>::new(n)
            .ok_or_else(|| anyhow!("Failed to create domain."))?;

        let vanishing_polynomial = DensePolynomial::from(domain.vanishing_polynomial());
        let vanishing_tau = commit::<G2Projective>(&pk_g2, &vanishing_polynomial)
            .map_err(|e| anyhow!("Failed to compute vanishing_tau with error: {e}."))?;

        Ok(Self {
            pk_g1,
            pk_g2,
            vanishing_tau,
            domain,
        })
    }

    fn build_public_keys(n: usize, tau: &Fr) -> (PublicKey<G1Affine>, PublicKey<G2Affine>) {
        let tau_i: Vec<Fr> = (0..n + 1)
            .into_par_iter()
            .map(|i| tau.pow([i as u64]))
            .collect();

        let g1_tau_i = G1Projective::generator().batch_mul(&tau_i);
        let g2_tau_i = G2Projective::generator().batch_mul(&tau_i);

        let pk_g1 = PublicKey::new(g1_tau_i);
        let pk_g2 = PublicKey::new(g2_tau_i);

        (pk_g1, pk_g2)
    }
}
