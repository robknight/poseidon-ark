use ark_bn254::Fr;
use ark_ff::fields::Field;
use ark_std::str::FromStr;
use ark_std::Zero;
use core::ops::{AddAssign, MulAssign};
use lazy_static::lazy_static;

#[cfg(feature = "poseidon-1")]
extern crate poseidon_1;
#[cfg(feature = "poseidon-2")]
extern crate poseidon_2;
#[cfg(feature = "poseidon-3")]
extern crate poseidon_3;
#[cfg(feature = "poseidon-4")]
extern crate poseidon_4;
#[cfg(feature = "poseidon-5")]
extern crate poseidon_5;
#[cfg(feature = "poseidon-6")]
extern crate poseidon_6;
#[cfg(feature = "poseidon-7")]
extern crate poseidon_7;
#[cfg(feature = "poseidon-8")]
extern crate poseidon_8;
#[cfg(feature = "poseidon-9")]
extern crate poseidon_9;
#[cfg(feature = "poseidon-10")]
extern crate poseidon_10;
#[cfg(feature = "poseidon-11")]
extern crate poseidon_11;
#[cfg(feature = "poseidon-12")]
extern crate poseidon_12;
#[cfg(feature = "poseidon-13")]
extern crate poseidon_13;
#[cfg(feature = "poseidon-14")]
extern crate poseidon_14;
#[cfg(feature = "poseidon-15")]
extern crate poseidon_15;
#[cfg(feature = "poseidon-16")]
extern crate poseidon_16;

mod constants;

#[derive(Debug, Clone)]
pub struct Constants {
    pub c: Vec<Vec<Fr>>,
    pub m: Vec<Vec<Vec<Fr>>>,
    pub n_rounds_f: usize,
    pub n_rounds_p: Vec<usize>,
}

lazy_static! {
    static ref POSEIDON_CONSTANTS: Constants = {
        let (c_str, m_str) = constants::constants();
        let mut c: Vec<Vec<Fr>> = Vec::new();
        for i in 0..c_str.len() {
            let mut cci: Vec<Fr> = Vec::new();
            for j in 0..c_str[i].len() {
                let b: Fr = Fr::from_str(c_str[i][j]).unwrap();
                cci.push(b);
            }
            c.push(cci);
        }
        let mut m: Vec<Vec<Vec<Fr>>> = Vec::new();
        for i in 0..m_str.len() {
            let mut mi: Vec<Vec<Fr>> = Vec::new();
            for j in 0..m_str[i].len() {
                let mut mij: Vec<Fr> = Vec::new();
                for k in 0..m_str[i][j].len() {
                    let b: Fr = Fr::from_str(m_str[i][j][k]).unwrap();
                    mij.push(b);
                }
                mi.push(mij);
            }
            m.push(mi);
        }
        Constants {
            c,
            m,
            n_rounds_f: 8,
            n_rounds_p: vec![
                56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
            ],
        }
    };
}

pub struct Poseidon {
    constants: Constants,
}
impl Poseidon {
    pub fn new() -> Poseidon {
        Poseidon {
            constants: POSEIDON_CONSTANTS.clone(),
        }
    }
    pub fn ark(&self, state: &mut Vec<Fr>, c: &[Fr], it: usize) {
        for i in 0..state.len() {
            state[i].add_assign(&c[it + i]);
        }
    }

    pub fn sbox(&self, n_rounds_f: usize, n_rounds_p: usize, state: &mut Vec<Fr>, i: usize) {
        if i < n_rounds_f / 2 || i >= n_rounds_f / 2 + n_rounds_p {
            for j in 0..state.len() {
                let aux = state[j];
                state[j] = state[j].square();
                state[j] = state[j].square();
                state[j].mul_assign(&aux);
            }
        } else {
            let aux = state[0];
            state[0] = state[0].square();
            state[0] = state[0].square();
            state[0].mul_assign(&aux);
        }
    }

    pub fn mix(&self, state: &Vec<Fr>, m: &[Vec<Fr>]) -> Vec<Fr> {
        let mut new_state: Vec<Fr> = Vec::new();
        for i in 0..state.len() {
            new_state.push(Fr::zero());
            for j in 0..state.len() {
                let mut mij = m[i][j];
                mij.mul_assign(&state[j]);
                new_state[i].add_assign(&mij);
            }
        }
        new_state.clone()
    }

    pub fn hash(&self, inp: Vec<Fr>) -> Result<Fr, String> {
        let t = inp.len() + 1;
        if inp.is_empty() {
            return Err("Empty input".to_string());
        }
        
        // Check if we have the constants for this input length
        let t_idx = t - 2;
        if t_idx >= self.constants.c.len() {
            return Err(format!("No constants available for input length {}. Enable the appropriate feature flag.", inp.len()));
        }

        let n_rounds_f = self.constants.n_rounds_f.clone();
        let n_rounds_p = self.constants.n_rounds_p[t_idx].clone();

        let mut state = vec![Fr::zero(); t];
        state[1..].clone_from_slice(&inp);

        for i in 0..(n_rounds_f + n_rounds_p) {
            self.ark(&mut state, &self.constants.c[t_idx], i * t);
            self.sbox(n_rounds_f, n_rounds_p, &mut state, i);
            state = self.mix(&state, &self.constants.m[t_idx]);
        }

        Ok(state[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_constants() {
        let cons = POSEIDON_CONSTANTS.clone();
        assert_eq!(
            cons.c[0][0].to_string(),
            "4417881134626180770308697923359573201005643519861877412381846989312604493735"
        );
        assert_eq!(
            cons.c[cons.c.len() - 1][0].to_string(),
            "21579410516734741630578831791708254656585702717204712919233299001262271512412"
        );
        assert_eq!(
            cons.m[0][0][0].to_string(),
            "2910766817845651019878574839501801340070030115151021261302834310722729507541"
        );
        assert_eq!(
            cons.m[cons.m.len() - 1][0][0].to_string(),
            "11497693837059016825308731789443585196852778517742143582474723527597064448312"
        );
    }

    #[test]
    fn test_hash() {
        let b0: Fr = Fr::from_str("0").unwrap();
        let b1: Fr = Fr::from_str("1").unwrap();
        let b2: Fr = Fr::from_str("2").unwrap();
        let b3: Fr = Fr::from_str("3").unwrap();
        let b4: Fr = Fr::from_str("4").unwrap();
        let b5: Fr = Fr::from_str("5").unwrap();
        let b6: Fr = Fr::from_str("6").unwrap();
        let b7: Fr = Fr::from_str("7").unwrap();
        let b8: Fr = Fr::from_str("8").unwrap();
        let b9: Fr = Fr::from_str("9").unwrap();
        let b10: Fr = Fr::from_str("10").unwrap();
        let b11: Fr = Fr::from_str("11").unwrap();
        let b12: Fr = Fr::from_str("12").unwrap();
        let b13: Fr = Fr::from_str("13").unwrap();
        let b14: Fr = Fr::from_str("14").unwrap();
        let b15: Fr = Fr::from_str("15").unwrap();
        let b16: Fr = Fr::from_str("16").unwrap();

        let poseidon = Poseidon::new();

        let big_arr: Vec<Fr> = vec![b1];
        let h = poseidon.hash(big_arr.clone()).unwrap();
        assert_eq!(
            h.to_string(),
            "18586133768512220936620570745912940619677854269274689475585506675881198879027"
        );

        let big_arr: Vec<Fr> = vec![b1, b2];
        let h = poseidon.hash(big_arr.clone()).unwrap();
        assert_eq!(
            h.to_string(),
            "7853200120776062878684798364095072458815029376092732009249414926327459813530"
        );

        let big_arr: Vec<Fr> = vec![b1, b2, b0, b0, b0];
        let h = poseidon.hash(big_arr.clone()).unwrap();
        assert_eq!(
            h.to_string(),
            "1018317224307729531995786483840663576608797660851238720571059489595066344487"
        );

        let big_arr: Vec<Fr> = vec![b1, b2, b0, b0, b0, b0];
        let h = poseidon.hash(big_arr.clone()).unwrap();
        assert_eq!(
            h.to_string(),
            "15336558801450556532856248569924170992202208561737609669134139141992924267169"
        );

        let big_arr: Vec<Fr> = vec![b3, b4, b0, b0, b0];
        let h = poseidon.hash(big_arr.clone()).unwrap();
        assert_eq!(
            h.to_string(),
            "5811595552068139067952687508729883632420015185677766880877743348592482390548"
        );

        let big_arr: Vec<Fr> = vec![b3, b4, b0, b0, b0, b0];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "12263118664590987767234828103155242843640892839966517009184493198782366909018"
        );

        let big_arr: Vec<Fr> = vec![b1, b2, b3, b4, b5, b6];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "20400040500897583745843009878988256314335038853985262692600694741116813247201"
        );

        let big_arr: Vec<Fr> = vec![b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "8354478399926161176778659061636406690034081872658507739535256090879947077494"
        );

        let big_arr: Vec<Fr> = vec![b1, b2, b3, b4, b5, b6, b7, b8, b9, b0, b0, b0, b0, b0];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "5540388656744764564518487011617040650780060800286365721923524861648744699539"
        );

        let big_arr: Vec<Fr> = vec![
            b1, b2, b3, b4, b5, b6, b7, b8, b9, b0, b0, b0, b0, b0, b0, b0,
        ];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "11882816200654282475720830292386643970958445617880627439994635298904836126497"
        );

        let big_arr: Vec<Fr> = vec![
            b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16,
        ];
        let h = poseidon.hash(big_arr).unwrap();
        assert_eq!(
            h.to_string(),
            "9989051620750914585850546081941653841776809718687451684622678807385399211877"
        );
    }
    #[test]
    fn test_wrong_inputs() {
        let b0: Fr = Fr::from_str("0").unwrap();
        let b1: Fr = Fr::from_str("1").unwrap();
        let b2: Fr = Fr::from_str("2").unwrap();

        let poseidon = Poseidon::new();

        let big_arr: Vec<Fr> = vec![
            b1, b2, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0, b0,
        ];
        poseidon.hash(big_arr).expect_err("Wrong inputs length");
    }
}
