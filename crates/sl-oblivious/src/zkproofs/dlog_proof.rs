use elliptic_curve::{subtle::ConstantTimeEq, Field};
use k256::{ProjectivePoint, Scalar};
use merlin::Transcript;
use rand::prelude::*;

use sl_mpc_mate::message::*;

use crate::utils::TranscriptProtocol;

/// Non-interactive Proof of knowledge of discrete logarithm with Fiat-Shamir transform.
#[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct DLogProof {
    /// Public point `t`.
    pub t: Opaque<ProjectivePoint, GR>,

    /// Challenge response
    pub s: Opaque<Scalar, PF>,
}

impl DLogProof {
    /// Prove knowledge of discrete logarithm.
    // TODO: Do we need merlin?
    pub fn prove<R: CryptoRng + RngCore>(
        x: &Scalar,
        base_point: &ProjectivePoint,
        transcript: &mut Transcript,
        rng: &mut R,
    ) -> Self {
        let r = Scalar::random(rng);
        let t = base_point * &r;
        let y = base_point * x;
        let c = Self::fiat_shamir(&y, &t, base_point, transcript);

        let s = r + c * x;

        Self {
            t: t.into(),
            s: s.into(),
        }
    }

    /// Verify knowledge of discrete logarithm.
    pub fn verify(
        &self,
        y: &ProjectivePoint,
        base_point: &ProjectivePoint,
        transcript: &mut Transcript,
    ) -> bool {
        let c = Self::fiat_shamir(y, &self.t, base_point, transcript);
        let lhs = base_point * &self.s.0;
        let rhs = self.t + y * &c;

        lhs.ct_eq(&rhs).into()
    }

    /// Get fiat-shamir challenge for Discrete log proof.
    pub fn fiat_shamir(
        y: &ProjectivePoint,
        t: &ProjectivePoint,
        base_point: &ProjectivePoint,
        transcript: &mut Transcript,
    ) -> Scalar {
        transcript.append_point(b"y", y);
        transcript.append_point(b"t", t);
        transcript.append_point(b"base-point", base_point);
        transcript.challenge_scalar(b"DLogProof-challenge")
    }
}

#[cfg(test)]
mod tests {
    use k256::{ProjectivePoint, Scalar};
    use merlin::Transcript;
    use rand::thread_rng;

    use super::DLogProof;

    #[test]
    pub fn test_dlog_proof() {
        use k256::{ProjectivePoint, Scalar};
        use merlin::Transcript;
        use rand::thread_rng;

        let mut rng = thread_rng();
        let mut transcript = Transcript::new(b"test-dlog-proof");

        let x = Scalar::generate_biased(&mut rng);
        let base_point = ProjectivePoint::GENERATOR;
        let y = base_point * x;

        let proof =
            DLogProof::prove(&x, &base_point, &mut transcript, &mut rng);

        let mut verify_transcript = Transcript::new(b"test-dlog-proof");

        assert!(proof.verify(&y, &base_point, &mut verify_transcript));
    }

    #[test]
    pub fn test_wrong_dlog_proof() {
        let mut rng = thread_rng();
        let mut transcript = Transcript::new(b"test-dlog-proof");

        let x = Scalar::generate_biased(&mut rng);
        let base_point = ProjectivePoint::GENERATOR;
        let wrong_scalar = Scalar::generate_biased(&mut rng);
        let y = base_point * x;

        let proof = DLogProof::prove(
            &wrong_scalar,
            &base_point,
            &mut transcript,
            &mut rng,
        );

        let mut verify_transcript = Transcript::new(b"test-dlog-proof");

        assert!(!proof.verify(&y, &base_point, &mut verify_transcript));
    }

    #[test]
    pub fn test_dlog_proof_fiat_shamir() {
        use k256::{ProjectivePoint, Scalar};
        use merlin::Transcript;

        let mut rng = thread_rng();
        let mut transcript = Transcript::new(b"test-dlog-proof");

        let x = Scalar::generate_biased(&mut rng);
        let base_point = ProjectivePoint::GENERATOR;
        let y = base_point * x;

        let proof =
            DLogProof::prove(&x, &base_point, &mut transcript, &mut rng);

        let mut verify_transcript = Transcript::new(b"test-dlog-proof-wrong");

        assert!(
            !proof.verify(&y, &base_point, &mut verify_transcript),
            "Proof should fail with wrong transcript"
        );
    }
}
