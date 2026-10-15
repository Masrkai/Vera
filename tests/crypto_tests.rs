use vera::processing::openssl_backend::{OpensslProcessor, OpensslNotFoundError};

#[test]
fn test_find_openssl() {
    // Depending on system environment, openssl may or may not be installed.
    // We can at least invoke it and assert the result is a Result<String, _>.
    let result = OpensslProcessor::find_openssl();
    match result {
        Ok(path) => {
            assert!(!path.is_empty());
        }
        Err(_) => {
            // Openssl not found on this specific runner container, which is fine,
            // but we can test error formatting.
            let err = OpensslNotFoundError;
            assert!(format!("{}", err).contains("openssl binary not found"));
        }
    }
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    if OpensslProcessor::find_openssl().is_ok() {
        let plaintext = "secret metadata message 123!";
        let passphrase = "strongpassword";

        let encrypted = OpensslProcessor::encrypt(plaintext, passphrase);
        assert!(encrypted.is_ok());
        let ciphertext = encrypted.unwrap();
        assert!(!ciphertext.is_empty());

        let decrypted = OpensslProcessor::decrypt(&ciphertext, passphrase);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), plaintext);
    }
}
