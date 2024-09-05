# WhatsKey-Trasparency

<p align="center">
  <img src="https://github.com/xerox0/WhatsKey-Transparency/blob/main/Documenti/Immagini/whatsapp-2842640_640.png" alt="Project logo" width="600px">
</p>
<p align="center">
  <strong></strong> The encryption behind the world's most popular messaging app.
</p>

WhatsKey-Transparency is a project based on:
1. WhatsApp Encryption Overview - technical white paper
   (https://www.whatsapp.com/security/WhatsApp-Security-Whitepaper.pdf)
2. About end-to-end encryption - Help center article (https://faq.whatsapp.com/820124435853543)
3. About security-code change notifications - Help center article (https://faq.whatsapp.com/1524220618005378)
4. Auditable key directory (AKD) implementation (https://github.com/facebook/akd)
5. PEPR 23 - WhatsApp Key Transparency - (https://www.usenix.org/conference/pepr23/presentation/lewi)

The objective of this project is to understand the functioning of the key transparency introduced by WhatsApp and use the AKD that Meta has made available to simulate the functioning of the WhatsApp key transparency. 
To do this, in addition to meta's AKD repository, we also use the API that meta has made available as a crate which we can find here https://docs.rs/akd/latest/akd/. Here are also explained the 4 features that you can test with this demo: publication, lookup proofs, Key History Proofs, Append-Only Proofs.

## DEMO
The developed demo allows you to use the 4 operations available in the WhatsApp crate, which otherwise are only defined by Meta.

Through the interactive menu you can decide to:
1. make a **Publication**;
2. Request a **Lookup proof**;
3. Request a **History key proof**;
4. Request an **Append-only proof**;

Therefore, the demo developed in this way allows you to simulate the WhatsApp infrastructure during key transparency.

NB. For ease of implementation at the moment the hash function used is sha256 instead of blake3 for the calculation of the hashes that will be inserted in the Merkle tree, as WhatsApp applies in reality.

To test the project you can ```git clone``` the repo, positioning inside ```akd-keytrasparency/src``` folder and type the following commands:

```bash
cargo build
cargo run
```

