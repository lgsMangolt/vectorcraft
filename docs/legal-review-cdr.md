# CDR interoperability: licences and narrow legal review

Reviewed on 2026-11-04 for an initial Brazilian use context. Informational engineering
review, not a legal opinion or a declaration of compliance. Other countries and future
company integrations require their own assessment.

## Components and provenance

- VectorCraft: MIT OR Apache-2.0; original copyright and licence files are retained.
- TraceDraw reader/model/SVG export: MIT OR Apache-2.0, immutable revision
  `b98a041a6e26ed925991bcab7a788a34812339a7`, used under its Apache-2.0 option. Attribution
  is recorded in `NOTICE`. Inspect the dependency's own licence and clean-room record:
  https://github.com/tracedraw/tracedraw/blob/b98a041a6e26ed925991bcab7a788a34812339a7/CLEANROOM.md
- ArtCraft logo assets have separate trademark terms. The eight files in `docs/brand`
  are removed from this fork, with their licence kept for the provenance record. Original
  authorship is credited in plain text. A new visual identity is future work.
- This feature uses open-source APIs and user-authorized local drawings, not proprietary
  executable code. It does not distribute CorelDRAW or require its installation.
- Do not publish customer drawings, embedded fonts or extracted previews as fixtures.
  Synthetic test documents are generated in code. Font embedding rights are separate.

## Selective Brazilian reference

The `leis-brasil` helper's software source was unavailable. Planalto also refused access.
The following provisions were checked instead in the official Chamber of Deputies
updated text of [Law 9,609/1998](https://www2.camara.leg.br/legin/fed/lei/1998/lei-9609-19-fevereiro-1998-364738-normaatualizada-pl.html):

- Articles 2 and 5: software protection and authorized derivatives matter; open-source
  licence grants are the basis for reuse here, with their conditions preserved.
- Article 6(IV): its integration exception is expressly limited to the integrator's
  exclusive use. It is not relied on as a general authorization to redistribute a
  proprietary program or sell a reconstructed product.
- Article 9: use is subject to licensing. The earlier local Corel conversion bridge and
  any future company-installed proprietary software remain subject to their own terms.

No legal conclusion about reverse engineering another vendor's installed program is
needed for this implementation. Pending before commercialization: branded distribution,
all transitive dependency notices, customer contracts and data handling, and jurisdiction-
specific review of any new plugin, target application or proprietary dependency. The
technical existence of a parser or a study tool does not settle those questions.
