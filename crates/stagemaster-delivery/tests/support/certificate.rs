//! Fresh short-lived test credentials. No certificate-store changes or disabled verification.
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    KeyUsagePurpose,
};
use std::sync::OnceLock;
pub struct Credentials {
    pub ca: Vec<u8>,
    pub certificate: Vec<u8>,
    pub key: Vec<u8>,
}
pub fn credentials() -> &'static Credentials {
    static KEYS: OnceLock<Credentials> = OnceLock::new();
    KEYS.get_or_init(|| {
        let now = time::OffsetDateTime::now_utc();
        let mut ca = CertificateParams::new(Vec::<String>::new()).unwrap();
        ca.not_before = now - time::Duration::days(1);
        ca.not_after = now + time::Duration::days(7);
        ca.distinguished_name
            .push(DnType::CommonName, "StageMaster isolated test CA");
        ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let ca_key = KeyPair::generate().unwrap();
        let ca_certificate = ca.self_signed(&ca_key).unwrap();
        let issuer = Issuer::new(ca, ca_key);
        let mut leaf =
            CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into()]).unwrap();
        leaf.not_before = now - time::Duration::days(1);
        leaf.not_after = now + time::Duration::days(7);
        leaf.distinguished_name
            .push(DnType::CommonName, "localhost");
        leaf.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        leaf.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
        let key = KeyPair::generate().unwrap();
        let certificate = leaf.signed_by(&key, &issuer).unwrap();
        Credentials {
            ca: ca_certificate.der().to_vec(),
            certificate: certificate.der().to_vec(),
            key: key.serialize_der(),
        }
    })
}
