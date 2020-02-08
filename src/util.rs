use ring::{digest, pbkdf2, rand::SecureRandom, rand::SystemRandom};
// use lazy_static
static DIGEST_ALG: &'static digest::Algorithm = &digest::SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub type Pass_Hash = [u8; CREDENTIAL_LEN];
const PBKDF2_ITER: u32 = 100_000;
lazy_static! {
    static ref SALT_RNG: SystemRandom = SystemRandom::new();
}

fn salt() -> [u8; CREDENTIAL_LEN] {
    let mut salt = [0u8; CREDENTIAL_LEN];
    SALT_RNG.fill(&mut salt).unwrap();
    salt
}

fn hash_password(password: &str, salt: &[u8; CREDENTIAL_LEN]) -> Pass_Hash {
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        DIGEST_ALG,
        PBKDF2_ITER,
        salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );
    pbkdf2_hash
}

fn verify_password(
    actual_pw_hash: Pass_Hash,
    salt: &[u8; CREDENTIAL_LEN],
    attempted_pw: &str,
) -> bool {
    pbkdf2::verify(
        DIGEST_ALG,
        PBKDF2_ITER,
        salt,
        attempted_pw.as_bytes(),
        &actual_pw_hash,
    )
    .is_ok()
}

#[cfg(test)]
mod util_tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let salt = salt();
        let hash = hash_password("foobar", &salt);

        let res = pbkdf2::verify(DIGEST_ALG, PBKDF2_ITER, &salt, "foobar".as_bytes(), &hash);
        assert!(res.is_ok())
    }

    #[test]
    fn test_verify() {
        let salt = salt();
        let hash = hash_password("foobar", &salt);
        assert!(verify_password(hash, &salt, "foobar"))
    }
}
