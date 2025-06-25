use crate::commit::commit;
use crate::setup::Setup;
use crate::util::{ark_de, ark_se};
use anyhow::anyhow;
use ark_bls12_381::{Bls12_381, Fr, G1Projective, G2Projective};
use ark_ec::pairing::Pairing;
use ark_ff::{Field, Zero};
use ark_poly::univariate::DensePolynomial;
use ark_poly::{DenseUVPolynomial, EvaluationDomain, Evaluations};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::SubAssign;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    r_tau: G1Projective,
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    h_tau: G1Projective,
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    r_hat_tau: G1Projective,
    #[serde(serialize_with = "ark_se", deserialize_with = "ark_de")]
    pub y: Fr,
}

pub trait Lvc {
    fn open(setup: &Setup, a: &[Fr], b: &[Fr]) -> anyhow::Result<Proof>;
    fn verify(setup: &Setup, c: &G1Projective, b: &[Fr], proof: &Proof) -> anyhow::Result<()>;
}

pub struct LagrangeLvc;

impl Lvc for LagrangeLvc {
    fn open(setup: &Setup, a: &[Fr], b: &[Fr]) -> anyhow::Result<Proof> {
        // SUM a_i * lambda_i(X)
        let poly_a = Evaluations::from_vec_and_domain(a.to_vec(), setup.domain).interpolate();
        // SUM b_i * lambda_i(X)
        let poly_b = Evaluations::from_vec_and_domain(b.to_vec(), setup.domain).interpolate();

        // (SUM a_i * lambda_i(X)) * (SUM b_i * lambda_i(X))
        let mut lhs = &poly_a * &poly_b;

        // (m^-1) * y
        let y = a.par_iter().zip(b).map(|(a, b)| a * b).sum::<Fr>();
        let m1 = Fr::from(a.len() as u64)
            .inverse()
            .ok_or_else(|| anyhow!("Failed to compute inverse."))?;
        let y_m1 = DensePolynomial::from_coefficients_slice(&[y * m1]);

        // ((SUM a_i * lambda_i(X)) * (SUM b_i * lambda_i(X))) / (m^-1) * y
        lhs.sub_assign(&y_m1);

        // lhs / t(X)
        let (h, r) = lhs.divide_by_vanishing_poly(setup.domain);
        let r: Vec<Fr> = r.par_iter().skip(1).cloned().collect();

        // R(tau)
        let r_tau = commit(&setup.pk_g1, &r)?;
        // H(tau)
        let h_tau = commit(&setup.pk_g1, &h)?;
        // R^(tau) = X^2 * R(X)
        let r_hat_tau = commit(&setup.pk_g1, &[vec![Fr::zero(); 2], r].concat())?;

        Ok(Proof {
            r_tau,
            h_tau,
            r_hat_tau,
            y,
        })
    }

    fn verify(setup: &Setup, c: &G1Projective, b: &[Fr], proof: &Proof) -> anyhow::Result<()> {
        // SUM b_i[lambda_i(tau)]2
        let b = Evaluations::from_vec_and_domain(b.to_vec(), setup.domain).interpolate();
        let c_b = commit::<G2Projective>(&setup.pk_g2, b.coeffs())?;

        let y = setup.pk_g1.pk[0]
            * (proof.y
                * Fr::from(setup.domain.size() as u64)
                    .inverse()
                    .ok_or_else(|| anyhow!("Failed to compute inverse."))?);

        // e(c_a, c_b) - e(m^-1 * y, 1) == e(R, 1) + e(H, t(tau))
        let e1_11 = Bls12_381::pairing(c, c_b);
        let e1_12 = Bls12_381::pairing(y, setup.pk_g2.pk[0]);

        // e(c_a, c_b) - e(m^-1 * y, 1)
        let e1_1 = e1_11 - e1_12;

        let e1_21 = Bls12_381::pairing(proof.r_tau, setup.pk_g2.pk[1]);
        let e1_22 = Bls12_381::pairing(proof.h_tau, setup.vanishing_tau);

        // e(R, 1) + e(H, t(tau))
        let e1_2 = e1_21 + e1_22;

        // e(R, tau2) == e(R_hat, 1)
        let e2_1 = Bls12_381::pairing(proof.r_tau, setup.pk_g2.pk[2]);
        let e2_2 = Bls12_381::pairing(proof.r_hat_tau, setup.pk_g2.pk[0]);

        if !(e1_1 == e1_2 && e2_1 == e2_2) {
            return Err(anyhow!("Verification failed."));
        }

        Ok(())
    }
}
