# NexVigilant WebMCP Hub Configuration License Header

## Purpose

This document defines the standard license header block to be included at
the top of every configuration file published by NexVigilant to WebMCP Hub
or any MCP marketplace. The header must appear verbatim in every published
config file, above any configuration keys.

---

## Standard Header Block (Copy Exactly As Shown)

```
# =============================================================================
# NexVigilant MCP Configuration
# =============================================================================
#
# Copyright (c) 2024-present Matthew Campion, PharmD / NexVigilant
# https://nexvigilant.com
#
# License: PolyForm Noncommercial License 1.0.0
# Full license text: https://nexvigilant.com/licenses/community
#
# PERMITTED: Personal use, academic research, non-commercial learning,
#            classroom instruction, non-profit organization use.
#
# NOT PERMITTED: Use by for-profit entities for business purposes,
#                commercial products or services, client-facing deployments,
#                any use that generates revenue or commercial value.
#
# COMMERCIAL LICENSING:
# For use in commercial products, enterprise deployments, or any for-profit
# context, a commercial license is required.
# Apply at: https://nexvigilant.com/licensing
# Contact:  licensing@nexvigilant.com
#
# ENTERPRISE INQUIRIES:
# For CROs, pharmaceutical companies, biotech firms, or health technology
# companies, contact our enterprise team:
# Email:    enterprise@nexvigilant.com
# Web:      https://nexvigilant.com/enterprise
#
# By downloading, configuring, or using this file you agree to be bound
# by the PolyForm Noncommercial License 1.0.0 and NexVigilant Terms of
# Use (https://nexvigilant.com/terms).
#
# =============================================================================
```

---

## Usage Instructions

### Required Placement

The header block must appear:
- As the first content in every `.json`, `.yaml`, `.toml`, or `.md`
  configuration file published to any MCP marketplace
- Before any schema declarations, server configuration, or tool definitions
- Using the comment syntax appropriate to the file format (see below)

### Format Adaptations by File Type

**JSON files** — Use a top-level `"_license"` key as the first key in the
root object, since JSON does not support comments:

```json
{
  "_license": {
    "copyright": "Copyright (c) 2024-present Matthew Campion, PharmD / NexVigilant",
    "license": "PolyForm Noncommercial License 1.0.0",
    "license_url": "https://nexvigilant.com/licenses/community",
    "commercial_licensing": "https://nexvigilant.com/licensing",
    "contact": "licensing@nexvigilant.com",
    "enterprise": "enterprise@nexvigilant.com",
    "permitted": "Personal, academic, non-commercial use only",
    "prohibited": "Commercial use, for-profit entities, revenue-generating deployments"
  },
  ...
}
```

**YAML files** — Use YAML comment block at top of file (as shown in the
standard header above).

**TOML files** — Use TOML comment block (`#` prefix) at top of file.

**Markdown files** — Include a "License" section as the first section of
the document.

### Version Tracking

When updating a configuration file, update the copyright year range in the
header to reflect the current year. Do not remove prior years.

Format: `Copyright (c) [first year]-present Matthew Campion, PharmD`

---

## WebMCP Hub Marketplace Listing Requirements

In addition to the file header, every NexVigilant tool published to WebMCP
Hub must include the following in the marketplace listing metadata:

**License field:** `PolyForm Noncommercial 1.0.0`

**Description suffix:** Append to every tool description:
> "Free for non-commercial use. Commercial license required for for-profit
> use. See nexvigilant.com/licensing."

**Tags to include:** `non-commercial`, `community-edition`, `license-required-commercial`

**External link:** Always include a link to `https://nexvigilant.com/licensing`
in the "More Information" or equivalent field.

---

## Enforcement Notes

The presence of this header in published files serves four legal functions:

1. **Constructive notice** — Users who download the file receive explicit
   notice of license terms before use, establishing that any violation is
   willful, not innocent.

2. **License acceptance trigger** — The Terms of Use (TERMS-OF-USE.md)
   establish that use of the file constitutes acceptance. The header makes
   this visible at point of use.

3. **Attribution tracing** — Copyright notice enables tracing of distributed
   copies back to NexVigilant for enforcement purposes.

4. **Commercial funnel entry point** — The enterprise contact information
   creates a visible conversion path from community use to commercial
   licensing without requiring the user to search for it.

---

## Header Maintenance

Review the header annually to ensure:
- Contact email addresses are current
- License URLs resolve correctly
- Copyright year range is accurate
- Commercial licensing URL reflects current pricing/process

Assign header maintenance to: licensing@nexvigilant.com
Review cadence: Annual (January)
```

---
