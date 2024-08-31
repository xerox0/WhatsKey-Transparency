use akd::{
    AkdLabel, AkdValue, Directory, EpochHash, HistoryParams, VerifyResult,
    client, auditor
};
use akd::ecvrf::HardCodedAkdVRF;
use akd::storage::memory::AsyncInMemoryDatabase;
use akd::storage::StorageManager;
use akd::errors::AkdError;
use tokio; // Assicurati che Tokio sia importato per eseguire codice asincrono

type Config = akd::WhatsAppV1Configuration;

#[tokio::main]
async fn main() -> Result<(), AkdError> {
    // 1. Inizializza il database in memoria
    let db = AsyncInMemoryDatabase::new();
    let storage_manager = StorageManager::new_no_cache(db);
    let vrf = HardCodedAkdVRF {};

    // 2. Crea una nuova directory
    let akd = Directory::<Config, _, _>::new(storage_manager, vrf).await?;
    println!("Directory creata.");

    // 3. **Publishing**: Inserisci nuove voci nella directory
    let entries = vec![
        (AkdLabel::from("first entry"), AkdValue::from("first value")),
        (AkdLabel::from("second entry"), AkdValue::from("second value")),
    ];

    let EpochHash(epoch1, root_hash1) = akd.publish(entries).await?;
    println!("Pubblicato epoch {} con root hash: {}", epoch1, hex::encode(&root_hash1));

    // 4. **Lookup Proofs**: Esegui una query e ottieni una prova di lookup
    let (lookup_proof, epoch_hash) = akd.lookup(AkdLabel::from("first entry")).await?;
    let public_key = akd.get_public_key().await?;

    let lookup_result = client::lookup_verify::<Config>(
        public_key.as_bytes(),
        epoch_hash.hash(),
        epoch_hash.epoch(),
        AkdLabel::from("first entry"),
        lookup_proof,
    )?;

    assert_eq!(
        lookup_result,
        VerifyResult {
            epoch: epoch1,
            version: 1,
            value: AkdValue::from("first value"),
        }
    );
    println!("Prova di lookup verificata con successo.");

    // 5. **History Proofs**: Pubblica una nuova versione e ottieni la storia
    let entries = vec![
        (AkdLabel::from("first entry"), AkdValue::from("updated value"))
    ];

    let EpochHash(epoch2, root_hash2) = akd.publish(entries).await?;
    println!("Pubblicato epoch {} con root hash: {}", epoch2, hex::encode(&root_hash2));

    let (history_proof, _) = akd.key_history(
        &AkdLabel::from("first entry"),
        HistoryParams::default(),
    ).await?;

    let key_history_result = client::key_history_verify::<Config>(
        public_key.as_bytes(),
        root_hash2,
        epoch2,
        AkdLabel::from("first entry"),
        history_proof,
        akd::HistoryVerificationParams::default(),
    )?;

    assert_eq!(
        key_history_result,
        vec![
            VerifyResult {
                epoch: epoch2,
                version: 2,
                value: AkdValue::from("updated value"),
            },
            VerifyResult {
                epoch: epoch1,
                version: 1,
                value: AkdValue::from("first value"),
            },
        ]
    );
    println!("Prova di storia verificata con successo.");

    // 6. **Append-Only Proofs**: Verifica la coerenza delle modifiche tra due epoch
    let audit_proof = akd.audit(epoch1, epoch2).await?;
    let audit_result = auditor::audit_verify::<Config>(
        vec![root_hash1, root_hash2],
        audit_proof,
    ).await;

    assert!(audit_result.is_ok());
    println!("Prova append-only verificata con successo.");

    Ok(())
}
