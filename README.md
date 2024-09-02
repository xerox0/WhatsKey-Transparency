# WhatsKet-Trasparency
<p align="center">
  <img src="https://github.com/xerox0/WhatsKey-Transparency/blob/main/Documenti/Immagini/whatsapp-2842640_640.png" alt="Project logo" width="200px">
</p>

WhatsKey-Transparency is a project based on:
1. WhatsApp Encryption Overview - technical white paper
   (https://www.whatsapp.com/security/WhatsApp-Security-Whitepaper.pdf)
2. About end-to-end encryption - Help center article (https://faq.whatsapp.com/820124435853543)
3. About security-code change notifications - Help center article (https://faq.whatsapp.com/1524220618005378)
4. Auditable key directory (AKD) implementation (https://github.com/facebook/akd)
5. PEPR 23 - WhatsApp Key Transparency - (https://www.usenix.org/conference/pepr23/presentation/lewi)

The objective of this project is to understand the functioning of the key transparency introduced by WhatsApp and use the AKD that Meta has made available to simulate the functioning of the WhatsApp key transparency. 
To do this, in addition to meta's AKD repository, we also use the API that meta has made available as a crate which we can find here https://docs.rs/akd/latest/akd/

To test the project you can ```git clone``` the repo, positioning inside ```akd-keytrasparency/src``` folder and type the following commands:
```bash
cargo build
cargo run
```
c