# Mr. Caveat fortune-ticket artwork provenance

Date: 2026-10-04 UTC.
Status: private review deliverables, revision 2. Hero and social approved by the requester; avatar replaced with the requester's supplied mechanically re-centered version. No repository, npm tarball or public-site integration was performed. The complete tree is retained as repository-ready review material; the user specified that it should not be included in the npm tarball and that the hero should be referenced by URL when integrated.

## Tools and generation

All creative generation, reframing, face/pose work and background extraction used the built-in OpenAI image_gen.imagegen tool (tool binding image_gen__imagegen). This was one coherent reference-linked generation session, with the social and avatar derived from the selected hero. No external image generator, stock image, programmatic drawing, repainting, text overlay or manual alpha removal was used.

The tool exposed neither the underlying model/version nor a random seed. Both are unavailable, rather than inferred. Exact normalized prompts are preserved verbatim in prompts/. Every raw generated PNG, including rejected iterations, is preserved in raw/. Raw filenames below map to the tool-returned basenames. The tool returned only image_url and output_hint; no generation parameters beyond the supplied prompt, reference paths and transparent_background were exposed.

## Inputs and identity

The exact requested character sheet was located and visually inspected. It contains the named Announcement pose and five palette circles. The later compact avatar was also inspected, but the sheet governed this task. Original reference bytes are not redistributed in this ZIP; identities and hashes establish which inputs were used without asserting their licensing history.

### Governing character sheet

- Filename: MrCaveatConcept.png
- Library identity: libfile_f5cdb3debf048191838725a4aa6bcd85
- File/source identity: file_00000000c35c81fda28db0dfe4f4def1
- Dimensions/mode: 1536 × 1024, RGB
- SHA-256: 2c19ffc75efecb106f819d2a64a6d500bfd890eaa3b4f49843da0e9f92277c2a
- Role: Used as generation reference; its Announcement pose, five swatches, turnaround and materials govern.

### Current profile avatar

- Filename: current-avatar.png
- Library identity: Not a Library reference
- File/source identity: get_orbit_profile retrieved by parent
- Dimensions/mode: 512 × 512, RGBA
- SHA-256: e75d22147ea71567c2291e6391a600d7304adf3c67d6e3bdbe3c1f9a24f00214
- Role: Inspected and used as secondary likeness reference for first hero only; did not override sheet proportions.

### Supporting main look

- Filename: Mr-Caveat-main-look.png
- Library identity: libfile_d8ad3b6e6c0481918feac7857e4156cc
- File/source identity: file_000000009de881fdb0567a9ef49fc0b2
- Dimensions/mode: 600 × 1215, RGBA
- SHA-256: 77e76be8914e092ce3931f0f24acebc959e2e76f8b05a877b8448f26d0228f4c
- Role: Inspected only; not passed to imagegen.

## Palette evidence

The five sheet swatches have subtle texture and no printed numeric color codes. The following are the per-channel median RGB values of 11×11-pixel patches centered at (1290,894), (1338,894), (1387,894), (1432,894), (1476,894) in the 1536×1024 governing sheet, respectively:

- Red: #AC2F28, RGB (172,47,40)
- Near-black: #282626, RGB (40,38,38)
- Ivory: #F4EADC, RGB (244,234,220)
- Gold: #D1A54F, RGB (209,165,79)
- Gray: #525051, RGB (82,80,81)

These exact sampled targets were supplied to generation. They identify the intended material palette, not a claim that every rendered pixel or every shaded material exactly equals those values. Shading, generation variation and final hero quantization change pixel colors. The sheet background sample at (600,150) was RGB (207,205,202), supplied as a background approximation.

## Generation receipt and selection log

1. Prompt: prompts/hero.txt
   - Input references: character sheet + current avatar
   - transparent_background: false
   - Tool output basename: exec-faa77817-233a-45fc-ad1c-d5769219a7ee.png
   - Preserved file: raw/hero-initial.png (1774×887, RGB)
   - SHA-256: 838754d81aaa4482eb7267e6115b595fdb8692c437200ccfb87c7e0161926a44
   - Selection: Rejected: highest black hat point touched/clipped top edge.

2. Prompt: prompts/hero-reframe.txt
   - Input references: hero-initial.png
   - transparent_background: false
   - Tool output basename: exec-cd75dd1a-556a-48a0-ba9e-88fed89e23b2.png
   - Preserved file: raw/hero-reframe-rejected.png (1774×887, RGB)
   - SHA-256: c49fdc2cc897dd8f14f3ab2ce2df26174f7719d4b2b9fc043f6e86a50bfdf40e
   - Selection: Rejected: zoom-out produced unwanted full-body/stubby proportions.

3. Prompt: prompts/hero-topfix.txt
   - Input references: hero-initial.png
   - transparent_background: false
   - Tool output basename: exec-75620970-60a2-47f4-9ab3-044f20040393.png
   - Preserved file: raw/hero-selected.png (1774×887, RGB)
   - SHA-256: edafb98ef8f1b65ff4c0bfba0c9de8cb2e2fd9d8daa8d9e424bf4b1b6b086d54
   - Selection: Selected hero, cropped through thighs with hat contained.

4. Prompt: prompts/avatar.txt
   - Input references: hero-selected.png + character sheet
   - transparent_background: true
   - Tool output basename: exec-206ce36d-9405-4d83-ac02-a2a979b6c93a.png
   - Preserved file: raw/avatar-initial.png (1254×1254, RGBA)
   - SHA-256: c41e6448c55dffd525df7714df92a6b20aeb66b8bf0623fc5fff75c906359440
   - Selection: Rejected framing: bells reached lateral edge and nose too low.

5. Prompt: prompts/avatar-reframe.txt
   - Input references: avatar-initial.png
   - transparent_background: true
   - Tool output basename: exec-b682d767-939c-4493-be79-7790b0d6ad0d.png
   - Preserved file: raw/avatar-reframe.png (1254×1254, RGBA)
   - SHA-256: f33ab29f665c291396ca81b55e70396b2ddb5360e687dd62cb0bcec26d1d76ec
   - Selection: Intermediate: entire silhouette in frame; nose still below center.

6. Prompt: prompts/social.txt
   - Input references: hero-selected.png
   - transparent_background: false
   - Tool output basename: exec-7ec3c13b-2cb3-4785-8911-f39ae005aa01.png
   - Preserved file: raw/social-selected.png (1774×887, RGB)
   - SHA-256: 65ecf1903190a1c779450ee29cca0006aaac3d7736e2305ec7848828889fb29a
   - Selection: Selected social, slightly more headroom.

7. Prompt: prompts/avatar-nosefix.txt
   - Input references: avatar-reframe.png
   - transparent_background: true
   - Tool output basename: exec-68407cc8-8af4-42a1-a8b8-aed707b1ef43.png
   - Preserved file: raw/avatar-selected.png (1254×1254, RGBA)
   - SHA-256: e76a9177ac9746f86ac254076d35e67ff1c42f216f867c08763e3821e69cac5c
   - Selection: Selected avatar with centered nose.

## Mechanical postprocessing

Software: Python 3.12.14, Pillow 12.3.0, installed libimagequant 2.18.0 (liq_version 21800, called through Python ctypes).

- Hero: selected 1774×887 RGB raw → 1600×800 RGB using Pillow LANCZOS; perceptual quantization with libimagequant, maximum 46 colors, quality bounds 0–100, speed 3, dithering level 1.0; indexed PNG optimized with zlib compression level 9. libimagequant reported quantization-quality score 85. Final is 386,207 bytes, below 400,000 bytes. The indexed palette includes lighting/shading colors, not merely five flat swatches. Fine dithering is visible at full pixel size; the lossless unquantized resized source is retained as raw/hero-resized-unquantized.png for future use.
- Initial avatar: selected 1254×1254 RGBA raw → 1024×1024 using Pillow LANCZOS. The generator's genuine alpha channel was retained. That delivered version is now archived as raw/avatar-delivered-v1.png.
- Revision 2 avatar: replaced byte-for-byte with requester-supplied mr-caveat-avatar-centered.png on 2026-10-04. The requester describes their postprocessing as translation and 94% uniform scaling of the existing artwork. No image regeneration or additional final-image transformation was performed in this update. The exact transformation script/resampler was not supplied. Independently measured meaningful silhouette bounds changed from 894×584 to 840×549, consistent with 94% within rounding. User source Library ID: libfile_73f9d2c3d368819184663f248e2c83c0; file ID: file_00000000957881fdb1486ab5537f603c; SHA-256: 118b8249f732c2a417d67a3ed8237160054ee233082687d884ba44c59124f837. The accompanying user comparison is preserved as proofs/avatar-circle-compare-user.png (Library libfile_87a5416129708191b1479ac85307e164).
- Social: selected 1774×887 RGB raw → 1280×640 using Pillow LANCZOS. Optimized PNG, compression level 9; no palette reduction.
- Hero and social contain an sRGB chunk with rendering intent 0. The requester-supplied revision 2 avatar is untagged (no explicit sRGB chunk or ICC profile); its RGB values are interpreted as sRGB, consistent with the original artwork. Its exact supplied bytes were preserved rather than silently re-encoded. The original delivered avatar had an explicit sRGB chunk. No colorimetric conversion or recalibration was performed.
- Proofs are mechanically resized copies at 700px width. Avatar white/dark-background composites are inspection proofs only; the final avatar remains transparent.
- Compression experiments using Pillow median-cut and ImageMagick were inspected but not selected. The final hero derives directly from libimagequant output. No artistic pixel edits occurred outside imagegen.

The libimagequant API signatures were checked against its official header: https://raw.githubusercontent.com/ImageOptim/libimagequant/2.17.0/libimagequant.h .

## Checks and limits

Full-size images and 700px-width proofs were visually inspected. Hero and social each show one figure, both hands holding one continuous ticket, readable FORTUNE AHEAD, dense fine-print texture continuing through the bottom crop, a visible profile wind-up key, contained hat points, warm-gray studio background and an asymmetric mildly befuddled/confident smile. Fine print is texture and carries no important readable message. No character name is added in the final pixels. Social has slightly more headroom than hero.

Avatar remains head and hat only. The revision 2 supplied artwork has alpha 0–255 and all borders fully transparent. At alpha >16 its bounds are [92,218,932,767], center (511.5,492.0), with maximum radius 420.685px around canvas center. Including every nonzero-alpha pixel, maximum radius is 422.298px. No alpha pixel lies outside the 512px circle, leaving at least 89.70px radial clearance. The previous delivered avatar's visible center was (511.5,378.5), 133.5px above canvas center; its alpha >16 maximum radius was 519.047px and 651 such pixels lay outside the circle. Thus nose centering alone did not ensure safe circular use. The new image was checked on white/dark circular backgrounds and at native 32px/16px sizes. Small-size proofs support icon inspection; they do not guarantee readability in every renderer. Original measured nose-center claims apply only to the archived v1 delivery, not the corrected avatar.

Machine checks establish dimensions, bytes, channel modes, alpha and PNG color declaration. Visual judgments establish the review suitability of composition and likeness, not copyright, trademark clearance or exact reproducibility. The user retains final approval before public integration.

## Final deliverable hashes

- mr-caveat-hero.png: 1600×800, P, 386,207 bytes; SHA-256 30be2dcc7b1f8b39dd59120640a6fcadd931fad4ec7391ecd90ced57c2379f1f
- mr-caveat-avatar.png: 1024×1024, RGBA, 556,635 bytes; SHA-256 118b8249f732c2a417d67a3ed8237160054ee233082687d884ba44c59124f837
- mr-caveat-social.png: 1280×640, RGB, 842,620 bytes; SHA-256 05a87844d18758ea7f9ee7cc069c7d6904fdac43559a7353d0f8022cec77e94c

## Licensing and rights

See ../LICENSE for the requested CC BY-SA 4.0 artwork grant and its scope. OpenAI's applicable terms and Creative Commons documentation were reviewed separately; source links and qualifications are preserved in LICENSE. This provenance is not proof of input ownership, copyrightability, exclusive ownership or non-infringement. Input rights and any rights in the underlying character require the rights holder's authority. CC licensing applies only to rights the licensor holds and can grant.

## Revision 2 audit trail

The requester explicitly approved swapping in the centered file (message Sentinel_5eb2ead82d0081919fe5256eb5bea218). All 26 entries in the original SHA256SUMS were independently verified before changes. Original generation candidates remain intact. The prior delivered avatar and v1 provenance/validation are retained for history. Current validation.json was rerun from actual final bytes, and current SHA256SUMS was rebuilt and checked for every packaged payload file. Hero and social final hashes are unchanged. The ZIP itself is checked for CRC errors and extracted-byte hash agreement with its manifest. No new imagegen call was made for this revision.
