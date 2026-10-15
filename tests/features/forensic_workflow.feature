Feature: Cryptographic Steganography and Forensics
  As a security-conscious user
  I want to encrypt a secret message and hide it inside an image
  So that I can securely transmit hidden information

  Scenario: Encrypting and embedding a secret message
    Given a secret plaintext message "Confidential data" and passphrase "secret123"
    When the message is encrypted using OpenSSL
    Then the resulting ciphertext is non-empty and valid
