#!/usr/bin/env python3
"""
knowledge-layer — code-map generator (codemap.py)

Deterministic, local, NO LLM. Uses tree-sitter via tree-sitter-language-pack
to extract top-level symbols and imports from source files, then emits:
  knowledge/wiki/_codemap.md   — agent-readable structural map
  knowledge/_codemap.json      — machine copy (modules, symbols, import edges)

Run via uv (zero permanent repo deps):
  uv run --with tree-sitter --with tree-sitter-language-pack \\
      python3 /path/to/codemap.py [repo-root]

Languages supported: rust, typescript, tsx, javascript, php, python
Unknown extensions are skipped gracefully.
If tree-sitter-language-pack is unavailable, the script exits with a clear
message rather than crashing — so the commit hook never fails a commit.
"""

import json
import os
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path

# ---------------------------------------------------------------------------
# Language registry — map file extension → tree-sitter grammar name
# ---------------------------------------------------------------------------
EXT_TO_LANG: dict[str, str] = {
    ".rs": "rust",
    ".ts": "typescript",
    ".tsx": "tsx",
    ".js": "javascript",
    ".mjs": "javascript",
    ".cjs": "javascript",
    ".php": "php",
    ".py": "python",
}

# Directories to always skip regardless of gitignore
SKIP_DIRS: set[str] = {
    "node_modules", "target", "dist", "build", ".git",
    "__pycache__", ".venv", "vendor", ".next", "out",
    "coverage", ".turbo",
}

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def get_text(node, source: bytes) -> str:
    """Slice byte-indexed node range from the source bytes, return as str."""
    return source[node.start_byte():node.end_byte()].decode("utf-8", errors="replace")


def first_child_of_kind(node, *kinds) -> object | None:
    for i in range(node.child_count()):
        c = node.child(i)
        if c.kind() in kinds:
            return c
    return None


def all_children_of_kind(node, *kinds) -> list:
    out = []
    for i in range(node.child_count()):
        c = node.child(i)
        if c.kind() in kinds:
            out.append(c)
    return out

# ---------------------------------------------------------------------------
# file discovery — respect .gitignore via git ls-files; fall back to walk
# ---------------------------------------------------------------------------

def collect_source_files(root: Path) -> list[Path]:
    """Return all source files under root, respecting .gitignore."""
    interesting_exts = set(EXT_TO_LANG.keys())

    # Use git ls-files when available — simplest + correct gitignore support.
    try:
        result = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
            cwd=root, capture_output=True, text=True, timeout=10
        )
        if result.returncode == 0:
            files = []
            for line in result.stdout.splitlines():
                p = root / line
                if p.suffix.lower() in interesting_exts and p.is_file():
                    # Still skip vendor dirs that git might track
                    parts = set(p.relative_to(root).parts)
                    if not parts & SKIP_DIRS:
                        files.append(p)
            return sorted(files)
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass

    # Fallback: walk, skipping SKIP_DIRS
    files = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            p = Path(dirpath) / fn
            if p.suffix.lower() in interesting_exts:
                files.append(p)
    return sorted(files)

# ---------------------------------------------------------------------------
# per-language symbol extraction
# ---------------------------------------------------------------------------

def extract_rust(node, source: bytes) -> tuple[list[str], list[str]]:
    """Return (symbols, imports) from a rust source_file node."""
    symbols, imports = [], []

    for i in range(node.child_count()):
        child = node.child(i)
        k = child.kind()

        if k == "use_declaration":
            # collect the path text (scoped_identifier or scoped_use_list)
            path_node = first_child_of_kind(child, "scoped_identifier", "scoped_use_list",
                                             "identifier", "use_wildcard")
            if path_node:
                imports.append(get_text(path_node, source))

        elif k == "function_item":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                name = get_text(name_node, source)
                # check visibility
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}fn {name}")

        elif k == "struct_item":
            name_node = first_child_of_kind(child, "type_identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}struct {name}")

        elif k == "enum_item":
            name_node = first_child_of_kind(child, "type_identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}enum {name}")

        elif k == "trait_item":
            name_node = first_child_of_kind(child, "type_identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}trait {name}")

        elif k == "impl_item":
            # "impl TypeName" or "impl Trait for TypeName"
            type_nodes = all_children_of_kind(child, "type_identifier",
                                              "generic_type", "scoped_type_identifier")
            if type_nodes:
                last = type_nodes[-1]
                impl_name = get_text(last, source)
                # Extract pub methods within the impl
                decl_list = first_child_of_kind(child, "declaration_list")
                if decl_list:
                    methods = []
                    for j in range(decl_list.child_count()):
                        m = decl_list.child(j)
                        if m.kind() == "function_item":
                            mn = first_child_of_kind(m, "identifier")
                            vis = first_child_of_kind(m, "visibility_modifier")
                            if mn:
                                mname = get_text(mn, source)
                                methods.append(("pub" if vis else "priv", mname))
                    pub_methods = [n for v, n in methods if v == "pub"]
                    if pub_methods:
                        symbols.append(f"impl {impl_name} [{', '.join(pub_methods)}]")
                    else:
                        symbols.append(f"impl {impl_name}")

        elif k == "const_item":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}const {name}")

        elif k == "type_alias":
            name_node = first_child_of_kind(child, "type_identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}type {name}")

        elif k == "mod_item":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                name = get_text(name_node, source)
                vis = first_child_of_kind(child, "visibility_modifier")
                prefix = "pub " if vis else ""
                symbols.append(f"{prefix}mod {name}")

    return symbols, imports


def _ts_export_name(export_stmt, source: bytes) -> str | None:
    """Extract the exported name from a TypeScript export_statement."""
    for i in range(export_stmt.child_count()):
        child = export_stmt.child(i)
        k = child.kind()

        if k in ("function_declaration", "generator_function_declaration"):
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                return f"export fn {get_text(name_node, source)}"

        elif k == "class_declaration":
            name_node = first_child_of_kind(child, "type_identifier", "identifier")
            if name_node:
                return f"export class {get_text(name_node, source)}"

        elif k == "interface_declaration":
            name_node = first_child_of_kind(child, "type_identifier", "identifier")
            if name_node:
                return f"export interface {get_text(name_node, source)}"

        elif k == "type_alias_declaration":
            name_node = first_child_of_kind(child, "type_identifier", "identifier")
            if name_node:
                return f"export type {get_text(name_node, source)}"

        elif k in ("lexical_declaration", "variable_declaration"):
            # const Foo = ... / let Foo = ...
            decl = first_child_of_kind(child, "variable_declarator")
            if decl:
                name_node = first_child_of_kind(decl, "identifier")
                if name_node:
                    return f"export const {get_text(name_node, source)}"

        elif k == "enum_declaration":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                return f"export enum {get_text(name_node, source)}"

    return None


def extract_ts_tsx(node, source: bytes) -> tuple[list[str], list[str]]:
    """Return (symbols, imports) from a typescript/tsx program node."""
    symbols, imports = [], []

    for i in range(node.child_count()):
        child = node.child(i)
        k = child.kind()

        if k == "import_statement":
            # grab the source string
            src_node = first_child_of_kind(child, "string")
            if src_node:
                raw = get_text(src_node, source).strip("'\"")
                imports.append(raw)

        elif k == "export_statement":
            name = _ts_export_name(child, source)
            if name:
                symbols.append(name)

        elif k in ("function_declaration", "class_declaration"):
            name_node = first_child_of_kind(child, "identifier", "type_identifier")
            if name_node:
                kind_word = "fn" if k == "function_declaration" else "class"
                symbols.append(f"{kind_word} {get_text(name_node, source)}")

    return symbols, imports


def extract_python(node, source: bytes) -> tuple[list[str], list[str]]:
    """Return (symbols, imports) from a python module node."""
    symbols, imports = [], []

    for i in range(node.child_count()):
        child = node.child(i)
        k = child.kind()

        if k == "import_statement":
            # import x, import x as y
            for j in range(child.child_count()):
                c = child.child(j)
                if c.kind() in ("dotted_name", "aliased_import"):
                    imports.append(get_text(c, source).split(" as ")[0].strip())

        elif k == "import_from_statement":
            # from x import y — track the module
            mod = first_child_of_kind(child, "dotted_name", "relative_import")
            if mod:
                imports.append(get_text(mod, source))

        elif k == "function_definition":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                symbols.append(f"def {get_text(name_node, source)}")

        elif k == "decorated_definition":
            # Look for the inner def/class
            inner = first_child_of_kind(child, "function_definition", "class_definition")
            if inner:
                name_node = first_child_of_kind(inner, "identifier")
                if name_node:
                    kind_word = "def" if inner.kind() == "function_definition" else "class"
                    symbols.append(f"{kind_word} {get_text(name_node, source)}")

        elif k == "class_definition":
            name_node = first_child_of_kind(child, "identifier")
            if name_node:
                symbols.append(f"class {get_text(name_node, source)}")

        elif k == "expression_statement":
            # top-level assignment: FOO = ...
            assign = first_child_of_kind(child, "assignment")
            if assign:
                lhs = first_child_of_kind(assign, "identifier")
                if lhs:
                    name = get_text(lhs, source)
                    if name.isupper():
                        symbols.append(f"const {name}")

    return symbols, imports


def extract_php(node, source: bytes) -> tuple[list[str], list[str]]:
    """Return (symbols, imports) from a php program node."""
    symbols, imports = [], []

    def walk(n):
        k = n.kind()
        if k in ("function_definition", "method_declaration"):
            name_node = first_child_of_kind(n, "name")
            if name_node:
                symbols.append(f"fn {get_text(name_node, source)}")
        elif k == "class_declaration":
            name_node = first_child_of_kind(n, "name")
            if name_node:
                symbols.append(f"class {get_text(name_node, source)}")
        elif k == "interface_declaration":
            name_node = first_child_of_kind(n, "name")
            if name_node:
                symbols.append(f"interface {get_text(name_node, source)}")
        elif k in ("require_expression", "require_once_expression",
                   "include_expression", "include_once_expression"):
            arg = first_child_of_kind(n, "string", "encapsed_string")
            if arg:
                imports.append(get_text(arg, source).strip("'\""))
        elif k == "namespace_use_declaration":
            # use Foo\Bar\Baz;
            for i in range(n.child_count()):
                c = n.child(i)
                if c.kind() == "namespace_use_clause":
                    imports.append(get_text(c, source).split(" as ")[0].strip())

        for i in range(n.child_count()):
            walk(n.child(i))

    walk(node)
    return symbols, imports


EXTRACTORS: dict[str, callable] = {
    "rust": extract_rust,
    "typescript": extract_ts_tsx,
    "tsx": extract_ts_tsx,
    "javascript": extract_ts_tsx,
    "python": extract_python,
    "php": extract_php,
}

# ---------------------------------------------------------------------------
# main analysis
# ---------------------------------------------------------------------------

def analyse_repo(root: Path) -> list[dict]:
    """Walk the repo and extract symbols+imports per file."""
    try:
        from tree_sitter_language_pack import get_parser
    except ImportError:
        print(
            "ERROR: tree-sitter-language-pack not found.\n"
            "Run via: uv run --with tree-sitter --with tree-sitter-language-pack "
            "python3 scripts/codemap.py [repo-root]",
            file=sys.stderr,
        )
        sys.exit(1)

    files = collect_source_files(root)
    results = []

    lang_parsers: dict[str, object] = {}

    for fpath in files:
        ext = fpath.suffix.lower()
        lang = EXT_TO_LANG.get(ext)
        if not lang:
            continue

        extractor = EXTRACTORS.get(lang)
        if not extractor:
            continue

        # lazy-load parser per language
        if lang not in lang_parsers:
            try:
                lang_parsers[lang] = get_parser(lang)
            except Exception as e:
                print(f"  skip {lang}: parser unavailable ({e})", file=sys.stderr)
                lang_parsers[lang] = None
        parser = lang_parsers[lang]
        if parser is None:
            continue

        try:
            source_bytes = fpath.read_bytes()
            # tree-sitter requires a string but returns byte offsets into the UTF-8
            # encoding — so we parse the decoded string and slice the raw bytes.
            source_str = source_bytes.decode("utf-8", errors="replace")
        except OSError:
            continue

        try:
            tree = parser.parse(source_str)
            root_node = tree.root_node()
            symbols, imports = extractor(root_node, source_bytes)
        except Exception as e:
            print(f"  parse error {fpath.relative_to(root)}: {e}", file=sys.stderr)
            symbols, imports = [], []

        rel = str(fpath.relative_to(root))
        results.append({
            "file": rel,
            "lang": lang,
            "symbols": symbols,
            "imports": sorted(set(imports)),
        })

    return results

# ---------------------------------------------------------------------------
# emitters
# ---------------------------------------------------------------------------

def group_by_dir(results: list[dict]) -> dict[str, list[dict]]:
    """Group file records by their immediate parent directory (or '.' for root)."""
    grouped: dict[str, list] = {}
    for r in results:
        parts = Path(r["file"]).parts
        dirkey = str(Path(*parts[:-1])) if len(parts) > 1 else "."
        grouped.setdefault(dirkey, []).append(r)
    return dict(sorted(grouped.items()))


def emit_markdown(results: list[dict], root: Path, regen_cmd: str) -> str:
    now = datetime.now().strftime("%Y-%m-%d %H:%M")
    total_files = len(results)
    total_symbols = sum(len(r["symbols"]) for r in results)
    langs_found = sorted(set(r["lang"] for r in results))

    lines = [
        "---",
        "name: _codemap",
        "type: meta",
        f"generated: {now}",
        "confidence: EXTRACTED",
        "do_not_edit: true",
        "---",
        "",
        "# Code Map — structural index (auto-generated, do not hand-edit)",
        "",
        "> **Auto-generated by tree-sitter (deterministic, no LLM). Confidence: EXTRACTED.**  ",
        f"> Languages: {', '.join(langs_found)}  ",
        f"> Files: {total_files} · Symbols: {total_symbols}  ",
        f"> Regenerate: `{regen_cmd}`  ",
        f"> Last generated: {now}",
        "",
        "Consult this map for \"how is X wired / where does Y live\" before grepping.",
        "Grep only to confirm or when the map is stale. The curated wiki/ADRs hold the WHY.",
        "",
        "---",
        "",
    ]

    grouped = group_by_dir(results)
    for dirkey, files in grouped.items():
        display_dir = dirkey if dirkey != "." else "(root)"
        lines.append(f"## {display_dir}")
        lines.append("")
        for r in sorted(files, key=lambda x: x["file"]):
            fname = Path(r["file"]).name
            lang = r["lang"]
            lines.append(f"### `{fname}` ({lang})")
            if r["imports"]:
                # Deduplicate and show concisely
                unique_imports = sorted(set(r["imports"]))
                # Trim long import lists
                if len(unique_imports) <= 6:
                    import_str = ", ".join(unique_imports)
                else:
                    import_str = ", ".join(unique_imports[:6]) + f" +{len(unique_imports)-6} more"
                lines.append(f"imports: {import_str}  ")
            if r["symbols"]:
                for sym in r["symbols"]:
                    lines.append(f"- {sym}")
            else:
                lines.append("- *(no top-level symbols extracted)*")
            lines.append("")

    return "\n".join(lines)


def emit_json(results: list[dict], root: Path) -> dict:
    now = datetime.now().isoformat(timespec="seconds")
    modules = []
    edges = []  # {from: file, to: import_path, kind: "import"}

    for r in results:
        modules.append({
            "file": r["file"],
            "lang": r["lang"],
            "symbols": r["symbols"],
            "imports": r["imports"],
        })
        for imp in r["imports"]:
            edges.append({"from": r["file"], "to": imp, "kind": "import"})

    return {
        "generated": now,
        "confidence": "EXTRACTED",
        "root": str(root),
        "modules": modules,
        "edges": edges,
    }

# ---------------------------------------------------------------------------
# entry point
# ---------------------------------------------------------------------------

def main():
    # Allow running as: uv run ... codemap.py [repo-root]
    repo_root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()

    if not repo_root.is_dir():
        print(f"ERROR: not a directory: {repo_root}", file=sys.stderr)
        sys.exit(1)

    knowledge_dir = repo_root / "knowledge"
    wiki_dir = knowledge_dir / "wiki"

    if not knowledge_dir.exists():
        print(
            f"ERROR: no knowledge/ directory at {repo_root}\n"
            "Run the knowledge-layer init.sh first.",
            file=sys.stderr,
        )
        sys.exit(1)

    wiki_dir.mkdir(parents=True, exist_ok=True)

    print(f"→ scanning {repo_root} ...", file=sys.stderr)
    results = analyse_repo(repo_root)

    if not results:
        print("  no source files found — nothing to map", file=sys.stderr)
        sys.exit(0)

    # Build the regen command (relative to repo root for portability)
    skill_path = Path(__file__).resolve()
    regen_cmd = f"uv run --with tree-sitter --with tree-sitter-language-pack python3 {skill_path} [repo-root]"

    # Emit markdown
    md_path = wiki_dir / "_codemap.md"
    md_content = emit_markdown(results, repo_root, regen_cmd)
    md_path.write_text(md_content, encoding="utf-8")

    # Emit JSON
    json_path = knowledge_dir / "_codemap.json"
    json_data = emit_json(results, repo_root)
    json_path.write_text(json.dumps(json_data, indent=2), encoding="utf-8")

    langs_found = sorted(set(r["lang"] for r in results))
    total_symbols = sum(len(r["symbols"]) for r in results)
    print(
        f"✓ {len(results)} files · {total_symbols} symbols · langs: {', '.join(langs_found)}",
        file=sys.stderr,
    )
    print(f"  → {md_path}", file=sys.stderr)
    print(f"  → {json_path}", file=sys.stderr)


if __name__ == "__main__":
    main()
