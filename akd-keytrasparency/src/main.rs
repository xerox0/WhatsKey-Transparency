use akd::{
    AkdLabel, AkdValue, Directory, EpochHash, HistoryParams,
    client, auditor
};
use akd::ecvrf::HardCodedAkdVRF;
use akd::storage::memory::AsyncInMemoryDatabase;
use akd::storage::StorageManager;
use akd::errors::AkdError;
use tokio;
use std::io::{self, Write};
use sha256::digest;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type Config = akd::WhatsAppV1Configuration;

#[tokio::main]
async fn main() -> Result<(), AkdError> {
    // Inizializza il database in memoria
    let db = AsyncInMemoryDatabase::new();
    let storage_manager = StorageManager::new_no_cache(db);
    let vrf = HardCodedAkdVRF {};

    // Crea una nuova directory
    let akd = Directory::<Config, _, _>::new(storage_manager, vrf).await?;
    println!("Directory creata.");

    let last_epoch = Arc::new(Mutex::new(None));
    let root_hashes = Arc::new(Mutex::new(HashMap::new()));

    loop {
        // Menu di scelta
        println!("\nScegli un'opzione:");
        println!("1. Aggiungi voci alla directory");
        println!("2. Verifica una prova di lookup");
        println!("3. Ottieni la storia di una chiave");
        println!("4. Verifica la prova di append-only");
        println!("5. Esci");
        print!("Inserisci la tua scelta: ");
        io::stdout().flush().unwrap();

        // Lettura dell'input dell'utente
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice: u32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Scelta non valida, riprova.");
                continue;
            }
        };

        match choice {
            1 => {
                // **Publishing**: Inserisci nuove voci nella directory
                print!("Inserisci una chiave (label) per l'inserimento: ");
                io::stdout().flush().unwrap();
                let mut label = String::new();
                io::stdin().read_line(&mut label).unwrap();
                let label = label.trim();

                let value = digest(label); // Usa SHA-256 per generare un valore univoco
                let entries = vec![
                    (AkdLabel::from(label), AkdValue::from(&value)),
                ];

                let EpochHash(epoch, root_hash) = akd.publish(entries).await?;
                println!("Pubblicato epoch {} con root hash: {}", epoch, hex::encode(&root_hash));

                // Memorizza l'ultimo epoch e root hash
                let mut last_epoch_lock = last_epoch.lock().unwrap();
                *last_epoch_lock = Some(epoch);

                let mut root_hashes_lock = root_hashes.lock().unwrap();
                root_hashes_lock.insert(epoch, root_hash);
            }
            2 => {
                // **Lookup Proofs**: Esegui una query e ottieni una prova di lookup
                print!("Inserisci l'etichetta da cercare: ");
                io::stdout().flush().unwrap();
                let mut label = String::new();
                io::stdin().read_line(&mut label).unwrap();
                let label = label.trim();

                match akd.lookup(AkdLabel::from(label)).await {
                    Ok((lookup_proof, epoch_hash)) => {
                        let public_key = akd.get_public_key().await?;

                        println!("Prova di lookup ottenuta.");
                        println!("Hash dell'epoch: {:?}", epoch_hash.hash());
                        println!("Epoch: {}", epoch_hash.epoch());

                        // Verifica la prova di lookup
                        match client::lookup_verify::<Config>(
                            public_key.as_bytes(),
                            epoch_hash.hash(),
                            epoch_hash.epoch(),
                            AkdLabel::from(label),
                            lookup_proof,
                        ) {
                            Ok(lookup_result) => {
                                println!("Prova di lookup verificata con successo.");
                                println!("Dettagli del risultato della verifica:");
                                println!("Epoch: {}", lookup_result.epoch);
                                println!("Versione: {}", lookup_result.version);
                                println!("Valore: {:?}", lookup_result.value);
                                println!("Valore previsto: {:?}", AkdValue::from(&digest(label)));
                            }
                            Err(e) => {
                                println!("Errore nella verifica della prova di lookup: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Errore nella ricerca dell'etichetta: {:?}", e);
                    }
                }
            }
            3 => {
                // **History Proofs**: Ottieni la storia di una chiave
                print!("Inserisci l'etichetta per ottenere la storia: ");
                io::stdout().flush().unwrap();
                let mut label = String::new();
                io::stdin().read_line(&mut label).unwrap();
                let label = label.trim();

                if let Some(epoch) = *last_epoch.lock().unwrap() {
                    match akd.key_history(
                        &AkdLabel::from(label),
                        HistoryParams::default(),
                    ).await {
                        Ok((history_proof, _)) => {
                            let public_key = akd.get_public_key().await?;

                            println!("Prova di storia ottenuta.");
                            println!("Root hash per la verifica della storia: {:?}", hex::encode(&root_hashes.lock().unwrap().get(&epoch).unwrap()));
                            println!("Epoch per la verifica della storia: {}", epoch);

                            match client::key_history_verify::<Config>(
                                public_key.as_bytes(),
                                root_hashes.lock().unwrap().get(&epoch).unwrap().clone(),
                                epoch,
                                AkdLabel::from(label),
                                history_proof,
                                akd::HistoryVerificationParams::default(),
                            ) {
                                Ok(key_history_result) => {
                                    println!("Prova di storia verificata con successo.");
                                    println!("Dettagli del risultato della storia:");
                                    for result in key_history_result {
                                        println!("Epoch: {}", result.epoch);
                                        println!("Versione: {}", result.version);
                                        println!("Valore: {:?}", result.value);
                                    }
                                }
                                Err(e) => {
                                    println!("Errore nella verifica della prova di storia: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Errore nel recupero della storia: {:?}", e);
                        }
                    }
                } else {
                    println!("Nessun dato disponibile per la storia. Assicurati di aver pubblicato almeno una voce.");
                }
            }
            4 => {
                // **Append-Only Proofs**: Verifica la coerenza delle modifiche tra due epoch
                print!("Inserisci l'epoch iniziale: ");
                io::stdout().flush().unwrap();
                let mut epoch1 = String::new();
                io::stdin().read_line(&mut epoch1).unwrap();
                let epoch1: u64 = match epoch1.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Epoch iniziale non valido.");
                        continue;
                    }
                };

                print!("Inserisci l'epoch finale: ");
                io::stdout().flush().unwrap();
                let mut epoch2 = String::new();
                io::stdin().read_line(&mut epoch2).unwrap();
                let epoch2: u64 = match epoch2.trim().parse() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("Epoch finale non valido.");
                        continue;
                    }
                };

                let root_hashes_lock = root_hashes.lock().unwrap();
                let root_hash1 = match root_hashes_lock.get(&epoch1) {
                    Some(hash) => hash.clone(),
                    None => {
                        println!("Root hash per epoch iniziale non trovato");
                        continue;
                    }
                };

                let root_hash2 = match root_hashes_lock.get(&epoch2) {
                    Some(hash) => hash.clone(),
                    None => {
                        println!("Root hash per epoch finale non trovato");
                        continue;
                    }
                };

                match akd.audit(epoch1, epoch2).await {
                    Ok(audit_proof) => {
                        println!("Prova di append-only ottenuta.");
                        println!("Root hash dell'epoch iniziale: {:?}", hex::encode(&root_hash1));
                        println!("Root hash dell'epoch finale: {:?}", hex::encode(&root_hash2));

                        match auditor::audit_verify::<Config>(
                            vec![root_hash1, root_hash2],
                            audit_proof,
                        ).await {
                            Ok(_) => {
                                println!("Prova append-only verificata con successo.");
                            }
                            Err(e) => {
                                println!("Errore nella verifica della prova append-only: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        // ...continuazione del codice...
                        println!("Errore nel recupero della prova append-only: {:?}", e);
                    }
                }
            }
            5 => {
                println!("Uscita in corso...");
                break;
            }
            _ => {
                println!("Scelta non valida, riprova.");
            }
        }
    }

    Ok(())
}

