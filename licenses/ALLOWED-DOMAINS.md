# NexVigilant Email Domain Access Policy
# Version 1.0 — Community Edition Free Tier

---

## Purpose

This document defines the email domain policy governing access to the
NexVigilant Community Edition (free tier). The policy exists to enforce the
non-commercial use restriction of the PolyForm Noncommercial License 1.0.0
at the point of account creation, before a user has agreed to any terms or
accessed any tools.

Email domain gating is a first-pass filter, not a complete enforcement
mechanism. It reduces administrative burden and routes commercial users to
the appropriate sales channel. The Terms of Use (TERMS-OF-USE.md) are the
governing legal instrument; this policy is an operational implementation.

---

## Section 1 — Explicit Allow List (Free Tier Access)

The following email domains and patterns are automatically approved for
free tier account creation. Users registering with these domains proceed
directly to the standard onboarding flow.

### 1.1 Personal Email Providers

These are treated as personal-use indicators. Users who register with
personal email addresses are still subject to the non-commercial use
restrictions and the prohibition on using personal email to circumvent
corporate access restrictions (see Section 4 and TERMS-OF-USE.md Section 3).

| Domain Pattern         | Provider          | Notes                        |
|------------------------|-------------------|------------------------------|
| `gmail.com`            | Google            | Largest personal email base  |
| `protonmail.com`       | Proton AG         | Privacy-focused              |
| `proton.me`            | Proton AG         | Alias domain                 |
| `outlook.com`          | Microsoft         | Personal accounts only       |
| `hotmail.com`          | Microsoft (legacy)| Personal accounts only       |
| `live.com`             | Microsoft (legacy)| Personal accounts only       |
| `yahoo.com`            | Yahoo             | Personal accounts            |
| `yahoo.co.uk`          | Yahoo (UK)        | Personal accounts            |
| `icloud.com`           | Apple             | Personal accounts            |
| `me.com`               | Apple (legacy)    | Personal accounts            |
| `mac.com`              | Apple (legacy)    | Personal accounts            |
| `hey.com`              | Basecamp          | Privacy-focused personal     |
| `fastmail.com`         | Fastmail          | Personal subscription        |
| `zoho.com`             | Zoho              | Personal tier only           |

### 1.2 Academic and University Domains

Academic domains are automatically approved. The pattern match is applied
to the full domain suffix after the `@` symbol.

| Pattern           | Coverage                                        | Examples                              |
|-------------------|-------------------------------------------------|---------------------------------------|
| `*.edu`           | US accredited educational institutions          | harvard.edu, unc.edu, purdue.edu      |
| `*.edu.au`        | Australian universities                         | unimelb.edu.au, anu.edu.au            |
| `*.edu.br`        | Brazilian federal universities                  | usp.edu.br                            |
| `*.edu.cn`        | Chinese universities                            |                                       |
| `*.edu.co`        | Colombian universities                          |                                       |
| `*.edu.in`        | Indian universities                             |                                       |
| `*.edu.mx`        | Mexican universities                            |                                       |
| `*.edu.sg`        | Singapore universities                          | nus.edu.sg                            |
| `*.edu.pk`        | Pakistani universities                          |                                       |
| `*.edu.za`        | South African universities                      |                                       |
| `*.ac.uk`         | UK universities and research councils           | ox.ac.uk, cam.ac.uk, ucl.ac.uk        |
| `*.ac.nz`         | New Zealand universities                        | auckland.ac.nz                        |
| `*.ac.jp`         | Japanese universities                           | u-tokyo.ac.jp                         |
| `*.ac.kr`         | South Korean universities                       |                                       |
| `*.ac.in`         | Indian autonomous institutions                  | iisc.ac.in                            |
| `*.ac.za`         | South African academic institutions             |                                       |
| `*.ac.il`         | Israeli universities                            | tau.ac.il                             |
| `*.ac.at`         | Austrian universities                           |                                       |
| `*.ac.be`         | Belgian universities                            |                                       |
| `*.ac.cn`         | Chinese research institutions                   |                                       |
| `*.uni-*.de`      | German universities (common pattern)            | uni-heidelberg.de                     |
| `*.tu-*.de`       | German technical universities                   | tu-berlin.de                          |
| `*.kit.edu`       | Karlsruhe Institute of Technology               |                                       |

**Note on academic domain edge cases:** Some universities use custom domains
that do not follow `.edu` or `.ac.*` patterns (e.g., `mit.edu` but also
standalone `polytechnique.fr`). Requests from unrecognized academic domains
should be routed to the manual review queue (Section 3) rather than
automatically blocked.

### 1.3 Government and Public Health Domains

Government researchers conducting non-commercial public health research
are permitted under the non-commercial use definition.

| Pattern           | Coverage                                        |
|-------------------|-------------------------------------------------|
| `*.gov`           | US federal government agencies                  |
| `*.gov.uk`        | UK government departments                       |
| `*.gc.ca`         | Canadian federal government                     |
| `*.gov.au`        | Australian federal government                   |
| `*.who.int`       | World Health Organization                       |
| `*.paho.org`      | Pan American Health Organization                |
| `*.ema.europa.eu` | European Medicines Agency                       |
| `*.fda.hhs.gov`   | US FDA (subset of *.gov)                        |

**Important limitation:** Government domains do not automatically confer
commercial use rights to contractors or vendors operating within those
agencies. A contractor using a `*.gov` email for work performed under a
commercial government contract must obtain a commercial license.

---

## Section 2 — Explicit Block List (Commercial Redirect)

The following domain patterns are blocked from free tier registration.
Users attempting to register with a blocked domain are redirected to
the enterprise signup flow at `https://nexvigilant.com/enterprise` with
a pre-populated form indicating their domain.

The block message displayed to the user is:

> "It looks like you're registering with a company email address.
> NexVigilant Community Edition is for non-commercial use only.
> For enterprise and commercial licensing, our team would love to
> help you find the right plan. [Contact Enterprise Team] →"

### 2.1 Pharmaceutical and Biotech Companies

The following domains are blocked. This list is not exhaustive — it
covers known major pharmaceutical companies. The catch-all rule in
Section 3 applies to companies not listed here.

**Big Pharma — Tier 1 (Revenue > $10B):**

| Company          | Domains to Block                                           |
|------------------|------------------------------------------------------------|
| Pfizer           | pfizer.com, pfizer.net, pfizerpro.com                      |
| Takeda           | takeda.com, takedapharm.com, shire.com                     |
| Novartis         | novartis.com, sandoz.com, alcon.com                        |
| Roche            | roche.com, genentech.com, chugai-pharm.co.jp               |
| Merck & Co (US)  | merck.com, msd.com, organon.com                            |
| Merck KGaA (DE)  | merckgroup.com, emdgroup.com, emd.com                      |
| Johnson & Johnson| jnj.com, janssen.com, janssenglobal.com                    |
| AbbVie           | abbvie.com, allergan.com                                   |
| Bristol-Myers Squibb | bms.com, celgene.com                                   |
| AstraZeneca      | astrazeneca.com, alexion.com, azdp.com                     |
| GlaxoSmithKline  | gsk.com, glaxo.com, gskpro.com, haleon.com                 |
| Sanofi           | sanofi.com, genzyme.com, synthelabo.com                    |
| Eli Lilly        | lilly.com, elililly.com                                    |
| Bayer            | bayer.com, bayerhealthcare.com                             |
| Amgen            | amgen.com                                                  |
| Gilead Sciences  | gilead.com                                                 |
| Biogen           | biogen.com                                                 |
| Regeneron        | regeneron.com                                              |
| Moderna          | modernatx.com, moderna.com                                 |
| BioNTech         | biontech.de, biontech.com                                  |
| Vertex           | vrtx.com, vrtxpharma.com                                   |
| Alexion          | alexion.com (also under astrazeneca.com)                   |
| Shire            | shire.com (also under takeda.com)                          |
| CSL Behring      | cslbehring.com, csl.com.au                                 |
| Novo Nordisk     | novonordisk.com, nn.com                                    |
| Boehringer Ingelheim | boehringer-ingelheim.com, boehringer.com              |
| UCB              | ucb.com                                                    |
| Ipsen            | ipsen.com                                                  |
| Jazz Pharma      | jazzpharma.com, jazzpharmaceuticals.com                    |
| Seagen           | seagen.com, seagen.net                                     |
| Incyte           | incyte.com                                                 |
| Alnylam          | alnylam.com                                                |
| Blueprint Medicines | blueprintmedicines.com                                  |

**Mid-Size and Specialty Pharma:**

| Company          | Domains to Block                                           |
|------------------|------------------------------------------------------------|
| Mylan/Viatris    | mylan.com, viatris.com                                     |
| Teva             | teva.com, tevapharmaceuticals.com                          |
| Sun Pharma       | sunpharma.com, sunpharmaceuticals.com                      |
| Dr. Reddy's      | drreddys.com                                               |
| Cipla            | cipla.com                                                  |
| Lupin            | lupin.com                                                  |
| Wockhardt        | wockhardt.com                                              |
| Daiichi Sankyo   | daiichi-sankyo.com, daiichisankyo.com                      |
| Astellas         | astellas.com                                               |
| Eisai            | eisai.com                                                  |
| Otsuka           | otsuka.co.jp, otsuka-us.com                                |
| Shionogi         | shionogi.com                                               |
| Sumitomo Pharma  | sumitomo-pharma.com, sumitomopharma.com                    |
| Ferring          | ferring.com                                                |
| Servier          | servier.com                                                |
| Pierre Fabre     | pierre-fabre.com                                           |
| Recordati        | recordati.com                                              |
| Menarini         | menarini.com                                               |
| Stallergenes     | stallergenes.com                                           |

### 2.2 Contract Research Organizations (CROs)

CROs operate commercially on behalf of pharmaceutical sponsors. All CRO
domains require commercial licensing regardless of the individual user's
role within the organization.

| Company          | Domains to Block                                           |
|------------------|------------------------------------------------------------|
| IQVIA            | iqvia.com, quintiles.com, imshealth.com                    |
| Covance          | covance.com, labcorp.com                                   |
| PPD              | ppdi.com, ppd.com, thermofisher.com (PPD subsidiary)       |
| PRA Health Sciences | prahs.com, prachealthsciences.com                       |
| Syneos Health    | syneoshealth.com, inventiv.com, imabc.com                  |
| Parexel          | parexel.com                                                |
| Icon plc         | iconplc.com, icon.com                                      |
| Medpace          | medpace.com                                                |
| Worldwide Clinical Trials | worldwide.com, worldwidemedical.com             |
| Clinipace        | clinipace.com                                              |
| Chiltern         | chiltern.com                                               |
| PSI CRO          | psicro.com                                                 |
| Novatek International | novatekintl.com                                      |

### 2.3 Health Technology and PV Software Companies

Companies offering competing or complementary commercial PV services.

| Company          | Domains to Block                                           |
|------------------|------------------------------------------------------------|
| Oracle Health Sciences | oracle.com (flag PV-related registrations)          |
| Veeva Systems    | veeva.com                                                  |
| Medidata         | medidata.com                                               |
| ERT              | ert.com                                                    |
| Bioclinica       | bioclinica.com                                             |
| BioClinica/Clario | clario.com                                               |
| ArisGlobal       | arisglobal.com                                             |
| UMC (Uppsala)    | who-umc.org, umc.uu.se (Note: non-commercial mission —    |
|                  | flag for manual review rather than hard block)             |

---

## Section 3 — Catch-All Rule for Unrecognized Domains

### 3.1 Logic

Any domain that:
- Does not match the explicit allow list (Section 1), AND
- Does not match the explicit block list (Section 2)

is treated as an **unrecognized domain** and triggers the manual review
flow.

### 3.2 Manual Review Flow

When a user registers with an unrecognized domain:

1. Account creation is paused (no access granted)
2. The user receives an automated email:

   > "Thank you for your interest in NexVigilant Community Edition.
   > We're reviewing your registration — we'll respond within 2 business
   > days. If you have a commercial or enterprise use case, you can reach
   > our team directly at enterprise@nexvigilant.com."

3. The registration is added to the manual review queue with the following
   information captured: email domain, stated purpose (from onboarding
   form), registration timestamp, country (from IP geolocation).

4. The reviewing team applies the following decision logic:

   - If the domain is clearly a personal domain not yet in the allow list:
     **Approve and add to allow list**
   - If the domain is a known company but not yet in the block list:
     **Block and add to block list; redirect to enterprise**
   - If the domain is ambiguous (e.g., a consultancy, staffing firm, or
     individual professional practice): **Request additional information**
   - If the stated purpose is academic but the domain is a company:
     **Deny free tier; offer educational institution pricing**

### 3.3 Allow List and Block List Updates

Updates to the allow and block lists are made by the NexVigilant
licensing team. The lists are version-controlled. Changes take effect
for new registrations immediately; existing accounts are not retroactively
affected by block list additions unless audit procedures under TERMS-OF-USE.md
Section 4 reveal active violations.

---

## Section 4 — Edge Cases and Policy Clarifications

### 4.1 Contractors Using Personal Email for Corporate Work

This is the most common circumvention vector. A consultant, contractor,
or freelancer who is performing paid pharmacovigilance services may
register with a personal Gmail or academic email address.

**Policy:** This is a Terms of Use violation, not a domain policy issue.
Domain gating cannot reliably detect this scenario. Coverage is provided
by TERMS-OF-USE.md Section 3 (Anti-Circumvention), which explicitly
prohibits using personal email to access the free tier while performing
work that constitutes commercial use. The audit and revocation rights in
TERMS-OF-USE.md Section 4 apply.

**Implementation note:** The onboarding questionnaire should include a
question asking users to confirm that their use is non-commercial and
not performed in connection with paid work for any for-profit entity.
This creates an affirmative representation that strengthens the ToS
claim in a dispute.

### 4.2 PhD Students at Industry-Sponsored Research Centers

PhD students whose research is funded by a pharmaceutical company but
who are enrolled at and operating under the auspices of an accredited
university are permitted to use the community edition under their
university email address, provided:
- Their output is for academic research purposes (thesis, publication)
- Results are not delivered directly to the sponsoring company as a
  work product
- Any resulting publications disclose the funding source per standard
  academic practice

If the student's primary deliverable is data or analysis for the
sponsoring company, that constitutes commercial use regardless of
the student's institutional affiliation.

### 4.3 Non-Profit Organizations with Government Grants

Non-profit organizations (501(c)(3) equivalents) operating under
government research grants qualify as non-commercial users even if
the grant is large. Registration requires a non-profit email domain
(e.g., `*.org`) or manual review confirmation of non-profit status.
Not all `.org` domains are non-profits — flag for manual review
unless the organization is well-known.

### 4.4 Hospital and Health System Researchers

Researchers at academic medical centers and teaching hospitals may
register if their use is for non-commercial clinical research.
Many hospital systems use `.org` or hospital-specific domains.
Flag for manual review. Key distinction:
- Research use by a physician-scientist: **Permitted**
- Operational use for a hospital's commercial PV department: **Not permitted**

### 4.5 Regulatory Affairs Consultants

Independent regulatory affairs consultants who use the tools for
client work are operating commercially, regardless of whether they
use a personal email address. This is covered by the anti-circumvention
clause in TERMS-OF-USE.md.

### 4.6 Students Doing Internships at Pharmaceutical Companies

An intern at a pharmaceutical company who registers with their personal
or university email address is not permitted to use the tools for intern
work at the company. They may use the tools for personal study outside
of company time, but company-related use constitutes commercial use.

---

## Section 5 — Policy Review and Maintenance

| Item                     | Responsibility              | Cadence       |
|--------------------------|-----------------------------|---------------|
| Block list updates       | Licensing team              | Quarterly     |
| Allow list updates       | Licensing team              | As needed     |
| Manual review queue      | Licensing team              | Within 2 biz days |
| Annual policy review     | Legal + Licensing           | Annual (Q1)   |
| Edge case documentation  | Legal                       | As cases arise|

Escalate novel edge cases to: licensing@nexvigilant.com
Legal escalation: [NexVigilant's legal counsel]

---

Policy version: 1.0
Effective date: [Date of first publication]
Next review: Q1 [Year + 1]
```

---
