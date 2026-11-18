"""Index a pinned upstream snapshot. Run from the repository root (Python 3.9+)."""
import argparse
from collections import Counter
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import re
import subprocess


OUT = Path("docs/coverage")
LOCK = OUT / "snapshot.json"
REPO = "ceifa/steamworks.js"
URL = "https://github.com/" + REPO


def run(*args):
    return subprocess.check_output(args, text=True).strip()


def git(*args):
    return run("git", *args)


def refs(namespace):
    return dict(line.split("\t") for line in git(
        "for-each-ref", "--format=%(refname:strip=2)%09%(objectname)", namespace
    ).splitlines())


def capture():
    dotgit = Path(git("rev-parse", "--git-dir"))
    forks = refs("refs/audit")
    branches = {}
    for line in (dotgit / "upstream-forks.tsv").read_text().splitlines():
        name, branch, _ = line.split("\t")
        if name in branches:
            raise ValueError(f"duplicate fork: {name}")
        branches[name] = branch
    if forks.keys() - branches.keys():
        raise ValueError("unexpected fork refs: refresh the fork list")
    pulls = {x["number"]: x for x in json.loads(run(
        "gh", "pr", "list", "-R", REPO, "--state", "all", "--limit", "10000",
        "--json", "number,state,title,headRefOid"
    ))}
    pr_refs = {int(k): v for k, v in refs("refs/pr").items()}
    for n, p in pulls.items():
        if p["state"] == "OPEN" and pr_refs.get(n) != p["headRefOid"]:
            raise ValueError(f"missing or stale open PR #{n}: fetch before capturing")
    issues = json.loads((dotgit / "upstream-issues.json").read_text())
    return {
        "upstream": git("rev-parse", "upstream/main"),
        "forks": [[n, branches[n], forks.get(n)] for n in sorted(branches)],
        "pulls": [[n, pr_refs[n], pulls.get(n, {}).get("state", "UNKNOWN"),
                   pulls.get(n, {}).get("title") or git("show", "-s", "--format=%s", pr_refs[n])]
                  for n in sorted(pr_refs)],
        "issues": sorted([x["number"], x["state"], x["title"]] for x in issues),
    }


# Each report gets one primary workstream; overlapping needs are noted in the plan.
ISSUES = {
    "Release and maintenance": "199 193 187 184 183 176 162 139 101 88 39 33",
    "Packaging and platform support": "206 194 192 188 173 156 153 149 136 131 129 126 123 122 121 120 103 93 91 83 80 75 71 68 67 64 62 61 51 46 45 42 38 26 19 18 14",
    "Overlay and rendering": "195 167 160 151 148 144 116 112 102 99 97 95 94 79 63 50 47 23 12",
    "Input and Deck controls": "202 185 166 154 135 108 56 53 76",
    "Workshop": "197 175 163 110 109 107 100 87 72",
    "Lifecycle, identity and callbacks": "171 165 138 134 128 127 119 118 111 105 77 65 60 59 57 54 52 48 24 22 21 16",
    "Stats and achievements": "170 150 132 89 86 70 69 15",
    "Friends and lobbies": "169 168 145 104 40",
    "Inventory": "164 152 55",
    "Networking": "124 90",
    "Cloud": "133 44 36 35",
    "Apps and documentation": "161 141 41 34 25 6 1",
}


FN = r"^\s*pub (?:async )?fn ([\w]+)\("


@lru_cache(None)
def upstream_functions(base):
    rows = git("grep", "-E", r"^[[:space:]]*pub (async )?fn ", base, "--", "src").splitlines()
    functions = set()
    for row in rows:
        _, path, declaration = row.split(":", 2)
        match = re.match(FN, declaration)
        if match:
            functions.add((path, match[1]))
    return functions


@lru_cache(None)
def inspect(base, tip):
    result = subprocess.run(["git", "merge-base", base, tip], text=True, capture_output=True)
    if result.returncode == 1:
        return "unrelated", None, [], []
    result.check_returncode()
    ancestor = result.stdout.strip()
    if ancestor == tip:
        return "upstream history", None, [], []
    raw = git("diff", "--raw", "--no-renames", "--no-abbrev", ancestor, tip)
    paths = [line.split("\t", 1)[1] for line in raw.splitlines()]
    fingerprint = hashlib.sha256(raw.encode()).hexdigest()[:16]
    diff = git("diff", "--no-ext-diff", "--no-renames", "-U0", ancestor, tip, "--", "src")
    added = set()
    path = None
    for line in diff.splitlines():
        if line.startswith("+++ b/"):
            path = line[6:]
        elif line.startswith("+") and (match := re.match(FN, line[1:])):
            added.add((path, match[1]))
    changed = sorted({name for _, name in added - upstream_functions(base)})
    return "delta" if raw else "same tree", fingerprint, paths, changed


def cell(text):
    return text.replace("|", r"\|").replace("\n", " ")


def render(snapshot):
    base = snapshot["upstream"]
    work = {}
    for topic, numbers in ISSUES.items():
        for n in map(int, numbers.split()):
            if n in work:
                raise ValueError(f"duplicate workstream: {n}")
            work[n] = topic
    issues = snapshot["issues"]
    if len(issues) != len({n for n, _, _ in issues}):
        raise ValueError("duplicate issues in snapshot")
    ids = {n for n, _, _ in issues}
    if work.keys() != ids:
        raise ValueError(f"update issue workstreams for {work.keys() ^ ids}")
    issue_rows = [f"| [#{n}]({URL}/issues/{n}) | {state} | {work[n]} | {cell(title)} |"
                  for n, state, title in issues]
    open_count = sum(s == "OPEN" for _, s, _ in issues)
    issues_md = [f"# Upstream issues\n\n{len(issues)} reports ({open_count} open). See the [plan](README.md) for disposition by workstream.",
                 "| Issue | State | Workstream | Report |", "|---|---|---|---|", *issue_rows]
    seen = {}
    fork_rows, pull_rows, signals = [], [], []
    counts = Counter()

    def delta(tip, label):
        status, key, paths, symbols = inspect(base, tip)
        if key and paths:
            first = seen.setdefault(key, label)
            if first == label:
                rust = [p for p in paths if p.startswith("src/")]
                other = Counter(p.split('/')[0] for p in paths if p not in rust)
                areas = ", ".join(f"`{p}`" + (f" ({n})" if n > 1 else "") for p, n in sorted(other.items()))
                signals.append(f"| {label} | {len(paths)} | {areas or 'none'} | {', '.join('`'+p+'`' for p in rust) or 'none'} | "
                               f"{', '.join('`'+s+'`' for s in symbols) or 'none'} |")
            else:
                return status, f"Same delta as {first}"
        return status, f"{status} ({len(paths)} paths)" if key else status

    for name, branch, tip in snapshot["forks"]:
        link = f"[{name}](https://github.com/{name}" + (f"/tree/{tip})" if tip else ")")
        status, finding = delta(tip, link) if tip else ("not fetched", "not fetched")
        counts[status] += 1
        fork_rows.append(f"| {link} | `{branch}` | {finding} |")
    for n, tip, state, title in snapshot["pulls"]:
        link = f"[#{n}]({URL}/pull/{n})"
        _, finding = delta(tip, link)
        pull_rows.append(f"| {link} | {state} | {cell(title)} | {finding} |")
    forks_md = ["# Fork default branches\n", ", ".join(f"{v} {k}" for k, v in sorted(counts.items())) + ".",
                "\nSee [diff evidence](deltas.md) for paths and names.",
                "| Fork at audited commit | Default branch | Change since merge base |", "|---|---|---|", *fork_rows]
    pr_md = ["# Upstream pull request refs\n", "PR state comes from the captured GitHub list, not Git ancestry. UNKNOWN means that list omitted the fetched ref.",
             "| PR | State | Title | Change since merge base |", "|---|---|---|---|", *pull_rows]
    signals_md = ["# Distinct fork and PR deltas\n", f"Base: [`{base[:12]}`]({URL}/commit/{base}). "
                  "One row per distinct nonempty diff from the tip's merge base. Equal deltas require identical paths, modes, and old/new blob IDs. "
                  "Rust names are public function declarations on added diff lines in `src/` whose path and name are absent from the pinned upstream. "
                  "A move or rename can look like a new API. These are not verified new JS exports. "
                  "Names miss fields, traits, callbacks, changed behavior and non-Rust implementations. Paths include non-API work.",
                  "| Source | Changed path count | Other files or directories | Changed paths under `src/` | Candidate function names |",
                  "|---|---:|---|---|---|", *signals]
    return {f"{name}.md": "\n".join(lines) + "\n" for name, lines in
            [("forks", forks_md), ("issues", issues_md), ("pulls", pr_md), ("deltas", signals_md)]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group()
    modes.add_argument("--capture", action="store_true", help="replace snapshot using staged audit refs and GitHub PR metadata")
    modes.add_argument("--check", action="store_true", help="compare tracked reports to a fresh offline render")
    args = parser.parse_args()
    snapshot = capture() if args.capture else json.loads(LOCK.read_text())
    reports = render(snapshot)
    if args.capture:
        LOCK.write_text(json.dumps(snapshot, ensure_ascii=False, indent=2) + "\n")
    for name, text in reports.items():
        path = OUT / name
        if args.check:
            if path.read_text() != text:
                raise SystemExit(f"stale audit: {path}")
        else:
            path.write_text(text)
    print(f"{len(snapshot['forks'])} forks, {len(snapshot['pulls'])} PR refs, {len(snapshot['issues'])} issues")
