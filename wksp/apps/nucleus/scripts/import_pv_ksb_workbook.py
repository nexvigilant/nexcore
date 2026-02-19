#!/usr/bin/env python3
"""Import PV KSB framework XLSX into normalized JSON using stdlib only."""

from __future__ import annotations

import argparse
import json
import re
import zipfile
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
import xml.etree.ElementTree as ET

NS_MAIN = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
NS_REL = "http://schemas.openxmlformats.org/officeDocument/2006/relationships"


def slug_key(text: str, fallback: str) -> str:
    value = (text or "").strip().lower()
    value = re.sub(r"[^a-z0-9]+", "_", value).strip("_")
    return value or fallback


def uniq_headers(headers: list[str]) -> list[str]:
    seen: dict[str, int] = {}
    out: list[str] = []
    for idx, h in enumerate(headers, start=1):
        base = slug_key(h, f"col_{idx}")
        count = seen.get(base, 0)
        if count:
            key = f"{base}_{count + 1}"
        else:
            key = base
        seen[base] = count + 1
        out.append(key)
    return out


def col_to_idx(cell_ref: str) -> int:
    col = "".join(ch for ch in cell_ref if ch.isalpha())
    n = 0
    for ch in col:
        n = n * 26 + (ord(ch.upper()) - 64)
    return n


def parse_workbook(xlsx_path: Path) -> dict:
    with zipfile.ZipFile(xlsx_path) as zf:
        wb_xml = ET.fromstring(zf.read("xl/workbook.xml"))
        rel_xml = ET.fromstring(zf.read("xl/_rels/workbook.xml.rels"))

        rel_map = {r.attrib["Id"]: r.attrib["Target"] for r in rel_xml}
        sheet_nodes = list(wb_xml.find(f"{{{NS_MAIN}}}sheets"))

        shared_strings: list[str] = []
        if "xl/sharedStrings.xml" in zf.namelist():
            ss_xml = ET.fromstring(zf.read("xl/sharedStrings.xml"))
            for si in ss_xml:
                text = "".join(t.text or "" for t in si.iter(f"{{{NS_MAIN}}}t"))
                shared_strings.append(text)

        def read_sheet_rows(sheet_target: str) -> list[dict[str, str]]:
            ws_xml = ET.fromstring(zf.read(f"xl/{sheet_target}"))
            data = ws_xml.find(f"{{{NS_MAIN}}}sheetData")
            if data is None:
                return []

            grid_rows: list[dict[int, str]] = []
            max_col = 0
            for row in data.findall(f"{{{NS_MAIN}}}row"):
                row_cells: dict[int, str] = {}
                for cell in row.findall(f"{{{NS_MAIN}}}c"):
                    ref = cell.attrib.get("r", "")
                    col_idx = col_to_idx(ref)
                    if col_idx <= 0:
                        continue
                    t = cell.attrib.get("t")
                    v = cell.find(f"{{{NS_MAIN}}}v")
                    is_node = cell.find(f"{{{NS_MAIN}}}is")
                    raw = ""
                    if v is not None and v.text is not None:
                        raw = v.text
                    elif is_node is not None:
                        raw = "".join(tn.text or "" for tn in is_node.iter(f"{{{NS_MAIN}}}t"))
                    else:
                        continue

                    if t == "s" and raw.isdigit():
                        val = shared_strings[int(raw)]
                    else:
                        val = raw
                    row_cells[col_idx] = val.strip()
                    max_col = max(max_col, col_idx)
                if row_cells:
                    grid_rows.append(row_cells)

            if not grid_rows:
                return []

            header_map = grid_rows[0]
            raw_headers = [header_map.get(i, "") for i in range(1, max_col + 1)]
            keys = uniq_headers(raw_headers)

            out_rows: list[dict[str, str]] = []
            for cells in grid_rows[1:]:
                row_obj: dict[str, str] = {}
                has_value = False
                for i in range(1, max_col + 1):
                    key = keys[i - 1]
                    val = cells.get(i, "").strip()
                    row_obj[key] = val
                    if val:
                        has_value = True
                if has_value:
                    out_rows.append(row_obj)
            return out_rows

        all_sheets: dict[str, list[dict[str, str]]] = {}
        sheet_names: list[str] = []
        for sn in sheet_nodes:
            name = sn.attrib.get("name", "Unnamed")
            rid = sn.attrib.get(f"{{{NS_REL}}}id")
            target = rel_map.get(rid, "")
            if not target:
                continue
            sheet_names.append(name)
            all_sheets[name] = read_sheet_rows(target)

        def pick(name: str) -> list[dict[str, str]]:
            return all_sheets.get(name, [])

        payload = {
            "source_file": str(xlsx_path),
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "sheet_names": sheet_names,
            "sheet_row_counts": {k: len(v) for k, v in all_sheets.items()},
            "domain_overview": pick("Domain Overview"),
            "capability_components": pick("Capability Components"),
            "epa_master": pick("EPA Master List"),
            "cpa_master": pick("CPA Master List"),
            "epa_domain_mapping": pick("EPA-Domain Mapping"),
            "cpa_domain_mapping": pick("CPA-Domain Mapping"),
            "cross_domain_integration": pick("Cross-Domain Integration"),
            "all_sheets": all_sheets,
        }
        return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("xlsx_path")
    parser.add_argument("output_json")
    args = parser.parse_args()

    xlsx_path = Path(args.xlsx_path)
    out_path = Path(args.output_json)

    payload = parse_workbook(xlsx_path)
    out_path.write_text(json.dumps(payload, ensure_ascii=True, indent=2), encoding="utf-8")

    print(f"wrote {out_path}")
    print(f"sheets={len(payload['sheet_names'])} capability_components={len(payload['capability_components'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
