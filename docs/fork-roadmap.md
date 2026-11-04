# Fork roadmap

Updated: 2026-11-04. This is a plan for this independent fork, not an upstream commitment.

1. **CDR import:** a separate Rust crate, immutable upstream reader dependency, bounded
   input, explicit loss notes and regression tests. Reach it through the existing engine
   so both people and agents use the same implementation. CDR writing is outside this step.
2. **Branding:** choose a product name, logo and application identity. ArtCraft trademark
   logo files are removed from this fork; original copyright and licence notices remain.
   The existing open-licensed application icon is temporary. Updates point to this fork,
   not upstream releases. Audit app ids,
   installers and documentation before distributing branded binaries.
3. **Company extensions:** build narrowly scoped adapters for real workflows (document
   preparation, print jobs, ERP/CRM access). The existing WASM plugin system supplies
   object filters, effects and exporters; it does not currently define a CDR importer ABI.
   Keep format decoding below the engine, then add an import-plugin contract when needed.
4. **Agents:** expose useful commands through the existing MCP/control API. Company data,
   credentials, file access, execution and printing permissions belong to each integration;
   never grant them implicitly because a plugin or agent exists. Receipt of a print job
   is separate from proof that a physical sheet came out correctly.
5. **Other tools:** use Craft Anything/REA to study authorized targets and gather evidence.
   Choose public specifications and reusable open-source components where possible.
   Evaluate licences and implementation scope for each tool; reconstruction is not an
   automatic guarantee of complete compatibility or permission to reuse branding/assets.

No company backend, login, remote-control pairing, WhatsApp monitor or unattended print
service is introduced by the CDR feature. Those require concrete workflow requirements.
